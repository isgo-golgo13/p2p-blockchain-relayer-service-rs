// core/blockchain-core/src/transaction.rs
use crate::{Address, Amount, Nonce, TxHash, Result, hash_serializable, validate_address, BlockchainError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Transaction types supported by the blockchain
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionType {
    /// Simple transfer between accounts
    Transfer {
        from: Address,
        to: Address,
        amount: Amount,
    },
    /// Smart contract deployment
    Deploy {
        from: Address,
        code: Vec<u8>,
        init_data: Vec<u8>,
    },
    /// Smart contract call
    Call {
        from: Address,
        to: Address,
        data: Vec<u8>,
        amount: Amount,
    },
}

/// Transaction status for tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionStatus {
    /// Pending in mempool
    Pending,
    /// Included in a block
    Confirmed { block_height: u64, block_hash: TxHash },
    /// Failed validation
    Failed { reason: String },
    /// Rejected by validation
    Rejected { reason: String },
}

/// Core transaction structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    /// Unique transaction hash
    pub hash: TxHash,
    /// Transaction type and data
    pub tx_type: TransactionType,
    /// Nonce to prevent replay attacks
    pub nonce: Nonce,
    /// Gas limit for execution
    pub gas_limit: u64,
    /// Gas price (fee per gas unit)
    pub gas_price: Amount,
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
    /// Digital signature
    pub signature: Vec<u8>,
    /// Current status
    pub status: TransactionStatus,
}

impl Transaction {
    /// Create a new transfer transaction
    pub fn new_transfer(
        from: Address,
        to: Address,
        amount: Amount,
        nonce: Nonce,
        gas_limit: u64,
        gas_price: Amount,
    ) -> Result<Self> {
        let tx_type = TransactionType::Transfer { from, to, amount };
        Self::new(tx_type, nonce, gas_limit, gas_price)
    }

    /// Create a new contract call transaction
    pub fn new_call(
        from: Address,
        to: Address,
        data: Vec<u8>,
        amount: Amount,
        nonce: Nonce,
        gas_limit: u64,
        gas_price: Amount,
    ) -> Result<Self> {
        let tx_type = TransactionType::Call { from, to, data, amount };
        Self::new(tx_type, nonce, gas_limit, gas_price)
    }

    /// Create a new contract deployment transaction
    pub fn new_deploy(
        from: Address,
        code: Vec<u8>,
        init_data: Vec<u8>,
        nonce: Nonce,
        gas_limit: u64,
        gas_price: Amount,
    ) -> Result<Self> {
        let tx_type = TransactionType::Deploy { from, code, init_data };
        Self::new(tx_type, nonce, gas_limit, gas_price)
    }

    /// Internal constructor
    fn new(
        tx_type: TransactionType,
        nonce: Nonce,
        gas_limit: u64,
        gas_price: Amount,
    ) -> Result<Self> {
        let timestamp = Utc::now();
        let signature = Vec::new(); // Will be filled by signing process
        let status = TransactionStatus::Pending;

        let mut tx = Transaction {
            hash: [0u8; 32], // Temporary hash
            tx_type,
            nonce,
            gas_limit,
            gas_price,
            timestamp,
            signature,
            status,
        };

        // Calculate actual hash
        tx.hash = tx.calculate_hash()?;
        Ok(tx)
    }

    /// Calculate transaction hash (excludes signature and status)
    pub fn calculate_hash(&self) -> Result<TxHash> {
        #[derive(Serialize)]
        struct HashableTransaction<'a> {
            tx_type: &'a TransactionType,
            nonce: Nonce,
            gas_limit: u64,
            gas_price: Amount,
            timestamp: DateTime<Utc>,
        }

        let hashable = HashableTransaction {
            tx_type: &self.tx_type,
            nonce: self.nonce,
            gas_limit: self.gas_limit,
            gas_price: self.gas_price,
            timestamp: self.timestamp,
        };

        hash_serializable(&hashable)
    }

    /// Get the sender address from the transaction
    pub fn sender(&self) -> Address {
        match &self.tx_type {
            TransactionType::Transfer { from, .. } => *from,
            TransactionType::Deploy { from, .. } => *from,
            TransactionType::Call { from, .. } => *from,
        }
    }

    /// Get the recipient address (if applicable)
    pub fn recipient(&self) -> Option<Address> {
        match &self.tx_type {
            TransactionType::Transfer { to, .. } => Some(*to),
            TransactionType::Call { to, .. } => Some(*to),
            TransactionType::Deploy { .. } => None,
        }
    }

    /// Get the amount being transferred
    pub fn amount(&self) -> Amount {
        match &self.tx_type {
            TransactionType::Transfer { amount, .. } => *amount,
            TransactionType::Call { amount, .. } => *amount,
            TransactionType::Deploy { .. } => 0,
        }
    }

    /// Calculate total transaction fee
    pub fn total_fee(&self) -> Amount {
        self.gas_limit * self.gas_price
    }

    /// Validate transaction structure
    pub fn validate_structure(&self) -> Result<()> {
        // Validate addresses
        match &self.tx_type {
            TransactionType::Transfer { from, to, amount } => {
                if !validate_address(from) {
                    return Err(BlockchainError::InvalidTransaction {
                        reason: "Invalid sender address".to_string(),
                    });
                }
                if !validate_address(to) {
                    return Err(BlockchainError::InvalidTransaction {
                        reason: "Invalid recipient address".to_string(),
                    });
                }
                if *amount == 0 {
                    return Err(BlockchainError::InvalidTransaction {
                        reason: "Transfer amount cannot be zero".to_string(),
                    });
                }
                if from == to {
                    return Err(BlockchainError::InvalidTransaction {
                        reason: "Cannot transfer to self".to_string(),
                    });
                }
            }
            TransactionType::Deploy { from, code, .. } => {
                if !validate_address(from) {
                    return Err(BlockchainError::InvalidTransaction {
                        reason: "Invalid deployer address".to_string(),
                    });
                }
                if code.is_empty() {
                    return Err(BlockchainError::InvalidTransaction {
                        reason: "Contract code cannot be empty".to_string(),
                    });
                }
            }
            TransactionType::Call { from, to, .. } => {
                if !validate_address(from) {
                    return Err(BlockchainError::InvalidTransaction {
                        reason: "Invalid caller address".to_string(),
                    });
                }
                if !validate_address(to) {
                    return Err(BlockchainError::InvalidTransaction {
                        reason: "Invalid contract address".to_string(),
                    });
                }
            }
        }

        // Validate gas parameters
        if self.gas_limit == 0 {
            return Err(BlockchainError::InvalidTransaction {
                reason: "Gas limit cannot be zero".to_string(),
            });
        }

        if self.gas_price == 0 {
            return Err(BlockchainError::InvalidTransaction {
                reason: "Gas price cannot be zero".to_string(),
            });
        }

        // Validate hash
        let calculated_hash = self.calculate_hash()?;
        if calculated_hash != self.hash {
            return Err(BlockchainError::InvalidTransaction {
                reason: "Transaction hash mismatch".to_string(),
            });
        }

        Ok(())
    }

    /// Update transaction status
    pub fn update_status(&mut self, status: TransactionStatus) {
        self.status = status;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_address(byte: u8) -> Address {
        [byte; 20]
    }

    #[test]
    fn test_transaction_creation() {
        let from = dummy_address(1);
        let to = dummy_address(2);
        let amount = 1000;
        let nonce = 1;
        let gas_limit = 21000;
        let gas_price = 20;

        let tx = Transaction::new_transfer(from, to, amount, nonce, gas_limit, gas_price).unwrap();
        
        assert_eq!(tx.sender(), from);
        assert_eq!(tx.recipient(), Some(to));
        assert_eq!(tx.amount(), amount);
        assert_eq!(tx.nonce, nonce);
        assert_eq!(tx.total_fee(), gas_limit * gas_price);
        assert_eq!(tx.status, TransactionStatus::Pending);
    }

    #[test]
    fn test_transaction_validation() {
        let from = dummy_address(1);
        let to = dummy_address(2);
        let tx = Transaction::new_transfer(from, to, 1000, 1, 21000, 20).unwrap();
        
        assert!(tx.validate_structure().is_ok());
    }

    #[test]
    fn test_invalid_transaction() {
        let from = dummy_address(1);
        let to = from; // Same address
        let result = Transaction::new_transfer(from, to, 1000, 1, 21000, 20);
        
        assert!(result.is_ok()); // Creation succeeds
        let tx = result.unwrap();
        assert!(tx.validate_structure().is_err()); // But validation fails
    }

    #[test]
    fn test_hash_consistency() {
        let from = dummy_address(1);
        let to = dummy_address(2);
        let tx = Transaction::new_transfer(from, to, 1000, 1, 21000, 20).unwrap();
        
        let hash1 = tx.calculate_hash().unwrap();
        let hash2 = tx.calculate_hash().unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(tx.hash, hash1);
    }
}
