// core/blockchain-core/src/block.rs
use crate::{Transaction, BlockHash, TxHash, BlockHeight, Result, hash_serializable, BlockchainError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Block header containing metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    /// Block height/index in the chain
    pub height: BlockHeight,
    /// Hash of the previous block
    pub previous_hash: BlockHash,
    /// Merkle root of all transactions in this block
    pub merkle_root: TxHash,
    /// Block timestamp
    pub timestamp: DateTime<Utc>,
    /// Nonce used for proof of work (if applicable)
    pub nonce: u64,
    /// Difficulty target for this block
    pub difficulty: u32,
    /// Version of the block format
    pub version: u32,
}

/// Complete block with header and transactions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    /// Block hash (calculated from header)
    pub hash: BlockHash,
    /// Block header
    pub header: BlockHeader,
    /// List of transactions in this block
    pub transactions: Vec<Transaction>,
    /// Transaction count for quick reference
    pub transaction_count: u32,
    /// Total size of the block in bytes
    pub size: u64,
}

impl Block {
    /// Create a new block
    pub fn new(
        height: BlockHeight,
        previous_hash: BlockHash,
        transactions: Vec<Transaction>,
        difficulty: u32,
    ) -> Result<Self> {
        let timestamp = Utc::now();
        let transaction_count = transactions.len() as u32;
        
        // Calculate merkle root from transactions
        let merkle_root = Self::calculate_merkle_root(&transactions)?;
        
        let header = BlockHeader {
            height,
            previous_hash,
            merkle_root,
            timestamp,
            nonce: 0, // Will be set during mining
            difficulty,
            version: 1,
        };

        let mut block = Block {
            hash: [0u8; 32], // Temporary
            header,
            transactions,
            transaction_count,
            size: 0, // Will be calculated
        };

        // Calculate actual hash and size
        block.hash = block.calculate_hash()?;
        block.size = block.calculate_size()?;

        Ok(block)
    }

    /// Create the genesis block (first block in chain)
    pub fn genesis() -> Result<Self> {
        let genesis_transactions = Vec::new();
        let previous_hash = [0u8; 32]; // No previous block
        let difficulty = 1; // Low difficulty for genesis
        
        Self::new(0, previous_hash, genesis_transactions, difficulty)
    }

    /// Calculate block hash from header
    pub fn calculate_hash(&self) -> Result<BlockHash> {
        hash_serializable(&self.header)
    }

    /// Calculate merkle root of transactions
    fn calculate_merkle_root(transactions: &[Transaction]) -> Result<TxHash> {
        if transactions.is_empty() {
            return Ok([0u8; 32]); // Empty merkle root
        }

        let mut hashes: Vec<TxHash> = transactions
            .iter()
            .map(|tx| tx.hash)
            .collect();

        // Build merkle tree bottom-up
        while hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in hashes.chunks(2) {
                let combined = if chunk.len() == 2 {
                    // Combine two hashes
                    let mut combined_data = Vec::new();
                    combined_data.extend_from_slice(&chunk[0]);
                    combined_data.extend_from_slice(&chunk[1]);
                    combined_data
                } else {
                    // Odd number - duplicate the last hash
                    let mut combined_data = Vec::new();
                    combined_data.extend_from_slice(&chunk[0]);
                    combined_data.extend_from_slice(&chunk[0]);
                    combined_data
                };
                
                let parent_hash = crate::hash_data(&combined_data);
                next_level.push(parent_hash);
            }
            
            hashes = next_level;
        }

        Ok(hashes[0])
    }

    /// Calculate the size of the block in bytes
    fn calculate_size(&self) -> Result<u64> {
        let serialized = bincode::serialize(self)?;
        Ok(serialized.len() as u64)
    }

    /// Validate the block structure and contents
    pub fn validate(&self) -> Result<()> {
        // Validate header hash
        let calculated_hash = self.calculate_hash()?;
        if calculated_hash != self.hash {
            return Err(BlockchainError::BlockValidationFailed {
                reason: "Block hash mismatch".to_string(),
            });
        }

        // Validate merkle root
        let calculated_merkle = Self::calculate_merkle_root(&self.transactions)?;
        if calculated_merkle != self.header.merkle_root {
            return Err(BlockchainError::BlockValidationFailed {
                reason: "Merkle root mismatch".to_string(),
            });
        }

        // Validate transaction count
        if self.transaction_count != self.transactions.len() as u32 {
            return Err(BlockchainError::BlockValidationFailed {
                reason: "Transaction count mismatch".to_string(),
            });
        }

        // Validate each transaction
        for tx in &self.transactions {
            tx.validate_structure()?;
        }

        // Validate timestamp (should not be too far in the future)
        let now = Utc::now();
        let max_future = now + chrono::Duration::minutes(10);
        if self.header.timestamp > max_future {
            return Err(BlockchainError::BlockValidationFailed {
                reason: "Block timestamp too far in future".to_string(),
            });
        }

        Ok(())
    }

    /// Check if this block can follow the given previous block
    pub fn can_follow(&self, previous_block: &Block) -> Result<()> {
        // Check height
        if self.header.height != previous_block.header.height + 1 {
            return Err(BlockchainError::BlockValidationFailed {
                reason: format!(
                    "Invalid height: expected {}, got {}",
                    previous_block.header.height + 1,
                    self.header.height
                ),
            });
        }

        // Check previous hash
        if self.header.previous_hash != previous_block.hash {
            return Err(BlockchainError::BlockValidationFailed {
                reason: "Previous hash mismatch".to_string(),
            });
        }

        // Check timestamp ordering
        if self.header.timestamp <= previous_block.header.timestamp {
            return Err(BlockchainError::BlockValidationFailed {
                reason: "Block timestamp must be after previous block".to_string(),
            });
        }

        Ok(())
    }

    /// Get total value of transactions in this block
    pub fn total_transaction_value(&self) -> u64 {
        self.transactions.iter().map(|tx| tx.amount()).sum()
    }

    /// Get total fees collected in this block
    pub fn total_fees(&self) -> u64 {
        self.transactions.iter().map(|tx| tx.total_fee()).sum()
    }

    /// Check if block contains a specific transaction
    pub fn contains_transaction(&self, tx_hash: &TxHash) -> bool {
        self.transactions.iter().any(|tx| &tx.hash == tx_hash)
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, tx_hash: &TxHash) -> Option<&Transaction> {
        self.transactions.iter().find(|tx| &tx.hash == tx_hash)
    }

    /// Set nonce (typically used during mining)
    pub fn set_nonce(&mut self, nonce: u64) -> Result<()> {
        self.header.nonce = nonce;
        self.hash = self.calculate_hash()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Transaction;

    fn dummy_address(byte: u8) -> crate::Address {
        [byte; 20]
    }

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis().unwrap();
        assert_eq!(genesis.header.height, 0);
        assert_eq!(genesis.header.previous_hash, [0u8; 32]);
        assert_eq!(genesis.transaction_count, 0);
        assert!(genesis.validate().is_ok());
    }

    #[test]
    fn test_block_with_transactions() {
        let tx1 = Transaction::new_transfer(
            dummy_address(1),
            dummy_address(2),
            1000,
            1,
            21000,
            20,
        ).unwrap();
        
        let tx2 = Transaction::new_transfer(
            dummy_address(3),
            dummy_address(4),
            2000,
            1,
            21000,
            20,
        ).unwrap();

        let transactions = vec![tx1, tx2];
        let block = Block::new(1, [1u8; 32], transactions, 1000).unwrap();
        
        assert_eq!(block.header.height, 1);
        assert_eq!(block.transaction_count, 2);
        assert_eq!(block.total_transaction_value(), 3000);
        assert!(block.validate().is_ok());
    }

    #[test]
    fn test_block_chain_validation() {
        let genesis = Block::genesis().unwrap();
        
        let tx = Transaction::new_transfer(
            dummy_address(1),
            dummy_address(2),
            1000,
            1,
            21000,
            20,
        ).unwrap();
        
        let block2 = Block::new(1, genesis.hash, vec![tx], 1000).unwrap();
        
        assert!(block2.can_follow(&genesis).is_ok());
    }

    #[test]
    fn test_merkle_root_calculation() {
        let tx1 = Transaction::new_transfer(
            dummy_address(1),
            dummy_address(2),
            1000,
            1,
            21000,
            20,
        ).unwrap();
        
        let transactions = vec![tx1];
        let merkle_root = Block::calculate_merkle_root(&transactions).unwrap();
        
        // Merkle root of single transaction should equal transaction hash
        assert_eq!(merkle_root, transactions[0].hash);
    }

    #[test]
    fn test_empty_merkle_root() {
        let merkle_root = Block::calculate_merkle_root(&[]).unwrap();
        assert_eq!(merkle_root, [0u8; 32]);
    }
}
