// storage/scylla-adapter/src/lib.rs
use anyhow::Result;
use blockchain_core::{Block, Transaction, Address, BlockHeight, TxHash, BlockHash};
use chrono::{DateTime, Utc};
use scylla::{Session, SessionBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub mod syclla_config;
pub mod scylla_queries;
pub mod dao;

use syclla_config::ScyllaConfig;
use dao::*;

/// Main ScyllaDB adapter for blockchain storage
pub struct ScyllaAdapter {
    session: Arc<Session>,
    config: ScyllaConfig,
    prepared_statements: Arc<RwLock<HashMap<String, scylla::prepared_statement::PreparedStatement>>>,
}

impl ScyllaAdapter {
    /// Create a new ScyllaDB adapter
    pub async fn new(config: ScyllaConfig) -> Result<Self> {
        let session = SessionBuilder::new()
            .known_nodes(&config.nodes)
            .user(&config.username, &config.password)
            .build()
            .await?;

        // Use the blockchain keyspace
        session.use_keyspace(&config.keyspace, false).await?;

        let adapter = ScyllaAdapter {
            session: Arc::new(session),
            config,
            prepared_statements: Arc::new(RwLock::new(HashMap::new())),
        };

        // Prepare commonly used statements
        adapter.prepare_statements().await?;

        Ok(adapter)
    }

    /// Prepare commonly used SQL statements for better performance
    async fn prepare_statements(&self) -> Result<()> {
        let mut statements = self.prepared_statements.write().await;

        // Block operations
        statements.insert(
            "insert_block".to_string(),
            self.session.prepare(queries::INSERT_BLOCK).await?,
        );
        statements.insert(
            "get_block_by_height".to_string(),
            self.session.prepare(queries::GET_BLOCK_BY_HEIGHT).await?,
        );
        statements.insert(
            "get_block_by_hash".to_string(),
            self.session.prepare(queries::GET_BLOCK_BY_HASH).await?,
        );

        // Transaction operations
        statements.insert(
            "insert_transaction".to_string(),
            self.session.prepare(queries::INSERT_TRANSACTION).await?,
        );
        statements.insert(
            "get_transaction".to_string(),
            self.session.prepare(queries::GET_TRANSACTION).await?,
        );
        statements.insert(
            "insert_tx_by_address".to_string(),
            self.session.prepare(queries::INSERT_TX_BY_ADDRESS).await?,
        );

        // Pending transactions
        statements.insert(
            "insert_pending_tx".to_string(),
            self.session.prepare(queries::INSERT_PENDING_TX).await?,
        );
        statements.insert(
            "delete_pending_tx".to_string(),
            self.session.prepare(queries::DELETE_PENDING_TX).await?,
        );

        // Account operations
        statements.insert(
            "update_account".to_string(),
            self.session.prepare(queries::UPDATE_ACCOUNT).await?,
        );
        statements.insert(
            "get_account".to_string(),
            self.session.prepare(queries::GET_ACCOUNT).await?,
        );

        Ok(())
    }

    /// Store a new block in the database
    pub async fn store_block(&self, block: &Block) -> Result<()> {
        let statements = self.prepared_statements.read().await;
        let stmt = statements
            .get("insert_block")
            .ok_or_else(|| anyhow::anyhow!("Insert block statement not prepared"))?;

        // Serialize the complete block
        let block_data = bincode::serialize(block)?;

        // Execute the insert
        self.session
            .execute(
                stmt,
                (
                    block.header.height as i64,
                    block.hash.to_vec(),
                    block.header.previous_hash.to_vec(),
                    block.header.merkle_root.to_vec(),
                    block.header.timestamp,
                    block.header.nonce as i64,
                    block.header.difficulty as i32,
                    block.header.version as i32,
                    block.transaction_count as i32,
                    block.size as i64,
                    block.total_transaction_value() as i64,
                    block.total_fees() as i64,
                    block_data,
                ),
            )
            .await?;

        // Also insert into hash index
        let hash_stmt = self.session.prepare(
            "INSERT INTO blocks_by_hash (hash, height) VALUES (?, ?)"
        ).await?;
        
        self.session
            .execute(&hash_stmt, (block.hash.to_vec(), block.header.height as i64))
            .await?;

        // Store all transactions in this block
        for (index, tx) in block.transactions.iter().enumerate() {
            self.store_transaction(tx, Some(block.header.height), Some(index as i32)).await?;
        }

        Ok(())
    }

    /// Retrieve a block by height
    pub async fn get_block_by_height(&self, height: BlockHeight) -> Result<Option<Block>> {
        let statements = self.prepared_statements.read().await;
        let stmt = statements
            .get("get_block_by_height")
            .ok_or_else(|| anyhow::anyhow!("Get block by height statement not prepared"))?;

        let rows = self.session.execute(stmt, (height as i64,)).await?;

        if let Some(row) = rows.first_row() {
            let block_data: Vec<u8> = row.columns[12].as_ref()
                .and_then(|col| col.as_blob())
                .ok_or_else(|| anyhow::anyhow!("Missing block data"))?
                .clone();

            let block: Block = bincode::deserialize(&block_data)?;
            Ok(Some(block))
        } else {
            Ok(None)
        }
    }

    /// Retrieve a block by hash
    pub async fn get_block_by_hash(&self, hash: &BlockHash) -> Result<Option<Block>> {
        // First get the height from hash index
        let hash_rows = self.session
            .query("SELECT height FROM blocks_by_hash WHERE hash = ?", (hash.to_vec(),))
            .await?;

        if let Some(row) = hash_rows.first_row() {
            let height: i64 = row.columns[0].as_ref()
                .and_then(|col| col.as_bigint())
                .ok_or_else(|| anyhow::anyhow!("Missing height"))?;

            self.get_block_by_height(height as BlockHeight).await
        } else {
            Ok(None)
        }
    }

    /// Store a transaction
    pub async fn store_transaction(
        &self, 
        tx: &Transaction, 
        block_height: Option<BlockHeight>,
        tx_index: Option<i32>
    ) -> Result<()> {
        let statements = self.prepared_statements.read().await;
        let stmt = statements
            .get("insert_transaction")
            .ok_or_else(|| anyhow::anyhow!("Insert transaction statement not prepared"))?;

        let tx_data = bincode::serialize(tx)?;
        let recipient_blob = tx.recipient().map(|addr| addr.to_vec());

        self.session
            .execute(
                stmt,
                (
                    tx.hash.to_vec(),
                    block_height.map(|h| h as i64),
                    tx_index,
                    tx.sender().to_vec(),
                    recipient_blob,
                    tx.amount() as i64,
                    format!("{:?}", tx.tx_type).split('{').next().unwrap_or("Unknown").to_string(),
                    tx.nonce as i64,
                    tx.gas_limit as i64,
                    tx.gas_price as i64,
                    tx.timestamp,
                    format!("{:?}", tx.status),
                    tx.signature.clone(),
                    tx_data,
                ),
            )
            .await?;

        // Add to transactions_by_address for sender
        self.add_transaction_to_address(&tx.sender(), tx, true).await?;

        // Add to transactions_by_address for recipient if exists
        if let Some(recipient) = tx.recipient() {
            self.add_transaction_to_address(&recipient, tx, false).await?;
        }

        // If part of a block, add to transactions_by_block
        if let (Some(height), Some(index)) = (block_height, tx_index) {
            self.session
                .query(
                    "INSERT INTO transactions_by_block (block_height, tx_index, tx_hash, timestamp) VALUES (?, ?, ?, ?)",
                    (height as i64, index, tx.hash.to_vec(), tx.timestamp),
                )
                .await?;
        }

        Ok(())
    }

    /// Add transaction to address index
    async fn add_transaction_to_address(
        &self,
        address: &Address,
        tx: &Transaction,
        is_sender: bool,
    ) -> Result<()> {
        let statements = self.prepared_statements.read().await;
        let stmt = statements
            .get("insert_tx_by_address")
            .ok_or_else(|| anyhow::anyhow!("Insert tx by address statement not prepared"))?;

        self.session
            .execute(
                stmt,
                (
                    address.to_vec(),
                    tx.timestamp,
                    tx.hash.to_vec(),
                    0i64, // block_height - will be updated when block is confirmed
                    format!("{:?}", tx.tx_type).split('{').next().unwrap_or("Unknown").to_string(),
                    tx.amount() as i64,
                    is_sender,
                ),
            )
            .await?;

        Ok(())
    }

    /// Add transaction to pending queue
    pub async fn add_pending_transaction(&self, tx: &Transaction) -> Result<()> {
        let statements = self.prepared_statements.read().await;
        let stmt = statements
            .get("insert_pending_tx")
            .ok_or_else(|| anyhow::anyhow!("Insert pending tx statement not prepared"))?;

        let priority_score = tx.gas_price * tx.gas_limit;
        let tx_data = bincode::serialize(tx)?;

        self.session
            .execute(
                stmt,
                (
                    tx.hash.to_vec(),
                    priority_score as i64,
                    tx.timestamp,
                    tx.sender().to_vec(),
                    tx.nonce as i64,
                    tx.gas_price as i64,
                    tx.gas_limit as i64,
                    tx_data,
                ),
            )
            .await?;

        Ok(())
    }

    /// Remove transaction from pending queue
    pub async fn remove_pending_transaction(&self, tx_hash: &TxHash) -> Result<()> {
        // First get the transaction to find priority_score and timestamp
        let rows = self.session
            .query(
                "SELECT priority_score, timestamp FROM pending_transactions WHERE tx_hash = ? ALLOW FILTERING",
                (tx_hash.to_vec(),),
            )
            .await?;

        if let Some(row) = rows.first_row() {
            let priority_score: i64 = row.columns[0].as_ref()
                .and_then(|col| col.as_bigint())
                .ok_or_else(|| anyhow::anyhow!("Missing priority_score"))?;
            let timestamp: DateTime<Utc> = row.columns[1].as_ref()
                .and_then(|col| col.as_timestamp())
                .ok_or_else(|| anyhow::anyhow!("Missing timestamp"))?;

            let statements = self.prepared_statements.read().await;
            let stmt = statements
                .get("delete_pending_tx")
                .ok_or_else(|| anyhow::anyhow!("Delete pending tx statement not prepared"))?;

            self.session
                .execute(stmt, (priority_score, timestamp, tx_hash.to_vec()))
                .await?;
        }

        Ok(())
    }

    /// Get pending transactions ordered by priority
    pub async fn get_pending_transactions(&self, limit: i32) -> Result<Vec<Transaction>> {
        let rows = self.session
            .query(
                "SELECT tx_data FROM pending_transactions LIMIT ?",
                (limit,),
            )
            .await?;

        let mut transactions = Vec::new();
        for row in rows.rows.unwrap_or_default() {
            if let Some(tx_data) = row.columns[0].as_ref().and_then(|col| col.as_blob()) {
                let tx: Transaction = bincode::deserialize(tx_data)?;
                transactions.push(tx);
            }
        }

        Ok(transactions)
    }

    /// Update account balance and nonce
    pub async fn update_account(
        &self,
        address: &Address,
        balance: u64,
        nonce: u64,
        account_type: &str,
    ) -> Result<()> {
        let statements = self.prepared_statements.read().await;
        let stmt = statements
            .get("update_account")
            .ok_or_else(|| anyhow::anyhow!("Update account statement not prepared"))?;

        self.session
            .execute(
                stmt,
                (
                    address.to_vec(),
                    balance as i64,
                    nonce as i64,
                    Utc::now(),
                    account_type.to_string(),
                    Option::<Vec<u8>>::None, // code_hash for contracts
                ),
            )
            .await?;

        Ok(())
    }

    /// Get account information
    pub async fn get_account(&self, address: &Address) -> Result<Option<AccountModel>> {
        let statements = self.prepared_statements.read().await;
        let stmt = statements
            .get("get_account")
            .ok_or_else(|| anyhow::anyhow!("Get account statement not prepared"))?;

        let rows = self.session.execute(stmt, (address.to_vec(),)).await?;

        if let Some(row) = rows.first_row() {
            let account = AccountModel {
                address: address.clone(),
                balance: row.columns[1].as_ref()
                    .and_then(|col| col.as_bigint())
                    .unwrap_or(0) as u64,
                nonce: row.columns[2].as_ref()
                    .and_then(|col| col.as_bigint())
                    .unwrap_or(0) as u64,
                last_updated: row.columns[3].as_ref()
                    .and_then(|col| col.as_timestamp())
                    .unwrap_or_else(Utc::now),
                account_type: row.columns[4].as_ref()
                    .and_then(|col| col.as_text())
                    .unwrap_or("user")
                    .to_string(),
                code_hash: row.columns[5].as_ref()
                    .and_then(|col| col.as_blob())
                    .map(|b| {
                        let mut hash = [0u8; 32];
                        if b.len() >= 32 {
                            hash.copy_from_slice(&b[..32]);
                        }
                        hash
                    }),
            };
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    /// Get transaction history for an address
    pub async fn get_address_transactions(
        &self,
        address: &Address,
        limit: i32,
    ) -> Result<Vec<AddressTransaction>> {
        let rows = self.session
            .query(
                "SELECT timestamp, tx_hash, block_height, tx_type, amount, is_sender FROM transactions_by_address WHERE address = ? LIMIT ?",
                (address.to_vec(), limit),
            )
            .await?;

        let mut transactions = Vec::new();
        for row in rows.rows.unwrap_or_default() {
            let tx = AddressTransaction {
                timestamp: row.columns[0].as_ref()
                    .and_then(|col| col.as_timestamp())
                    .ok_or_else(|| anyhow::anyhow!("Missing timestamp"))?,
                tx_hash: {
                    let hash_vec = row.columns[1].as_ref()
                        .and_then(|col| col.as_blob())
                        .ok_or_else(|| anyhow::anyhow!("Missing tx_hash"))?;
                    let mut hash = [0u8; 32];
                    if hash_vec.len() >= 32 {
                        hash.copy_from_slice(&hash_vec[..32]);
                    }
                    hash
                },
                block_height: row.columns[2].as_ref()
                    .and_then(|col| col.as_bigint())
                    .map(|h| h as u64),
                tx_type: row.columns[3].as_ref()
                    .and_then(|col| col.as_text())
                    .unwrap_or("Unknown")
                    .to_string(),
                amount: row.columns[4].as_ref()
                    .and_then(|col| col.as_bigint())
                    .unwrap_or(0) as u64,
                is_sender: row.columns[5].as_ref()
                    .and_then(|col| col.as_boolean())
                    .unwrap_or(false),
            };
            transactions.push(tx);
        }

        Ok(transactions)
    }

    /// Get latest block height
    pub async fn get_latest_block_height(&self) -> Result<Option<BlockHeight>> {
        let rows = self.session
            .query("SELECT height FROM blocks LIMIT 1", ())
            .await?;

        if let Some(row) = rows.first_row() {
            let height = row.columns[0].as_ref()
                .and_then(|col| col.as_bigint())
                .map(|h| h as BlockHeight);
            Ok(height)
        } else {
            Ok(None)
        }
    }

    /// Get chain statistics
    pub async fn get_chain_stats(&self) -> Result<ChainStats> {
        // Get latest block info
        let latest_height = self.get_latest_block_height().await?.unwrap_or(0);
        
        // Get total transaction count (this is an approximation)
        let tx_rows = self.session
            .query("SELECT COUNT(*) FROM transactions", ())
            .await?;
        
        let total_transactions = tx_rows.first_row()
            .and_then(|row| row.columns[0].as_ref())
            .and_then(|col| col.as_bigint())
            .unwrap_or(0) as u64;

        Ok(ChainStats {
            total_blocks: latest_height + 1,
            total_transactions,
            latest_block_height: latest_height,
            // Other stats would require more complex queries
            avg_block_time: 12.0, // Default value
            network_hash_rate: 0,
            active_addresses: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blockchain_core::Transaction;

    fn dummy_address(byte: u8) -> Address {
        [byte; 20]
    }

    // Note: These tests require a running ScyllaDB instance
    // Run with: cargo test --features integration-tests

    #[tokio::test]
    #[ignore] // Requires ScyllaDB setup
    async fn test_store_and_retrieve_block() {
        let config = ScyllaConfig::default();
        let adapter = ScyllaAdapter::new(config).await.unwrap();
        
        let block = Block::genesis().unwrap();
        adapter.store_block(&block).await.unwrap();
        
        let retrieved = adapter.get_block_by_height(0).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().hash, block.hash);
    }

    #[tokio::test]
    #[ignore] // Requires ScyllaDB setup
    async fn test_pending_transactions() {
        let config = ScyllaConfig::default();
        let adapter = ScyllaAdapter::new(config).await.unwrap();
        
        let tx = Transaction::new_transfer(
            dummy_address(1),
            dummy_address(2),
            1000,
            1,
            21000,
            20,
        ).unwrap();
        
        adapter.add_pending_transaction(&tx).await.unwrap();
        let pending = adapter.get_pending_transactions(10).await.unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].hash, tx.hash);
        
        adapter.remove_pending_transaction(&tx.hash).await.unwrap();
        let pending = adapter.get_pending_transactions(10).await.unwrap();
        assert_eq!(pending.len(), 0);
    }
}
