// core/blockchain-core/src/lib.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

pub mod block;
pub mod transaction;
pub mod chain;
pub mod merkle;

// Re-export main types
pub use block::*;
pub use transaction::*;
pub use chain::*;
pub use merkle::*;

/// Block hash type
pub type BlockHash = [u8; 32];

/// Transaction hash type  
pub type TxHash = [u8; 32];

/// Address type for accounts
pub type Address = [u8; 20];

/// Balance amount
pub type Amount = u64;

/// Block height/index
pub type BlockHeight = u64;

/// Nonce for transactions
pub type Nonce = u64;

/// Core blockchain errors
#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Invalid block hash: {0:?}")]
    InvalidBlockHash(BlockHash),
    
    #[error("Invalid transaction: {reason}")]
    InvalidTransaction { reason: String },
    
    #[error("Block validation failed: {reason}")]
    BlockValidationFailed { reason: String },
    
    #[error("Chain validation failed: {reason}")]
    ChainValidationFailed { reason: String },
    
    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: Amount, need: Amount },
    
    #[error("Invalid nonce: expected {expected}, got {actual}")]
    InvalidNonce { expected: Nonce, actual: Nonce },
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub type Result<T> = std::result::Result<T, BlockchainError>;

/// Functions for hashing
pub fn hash_data(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Generate a hash from serializable data
pub fn hash_serializable<T: Serialize>(data: &T) -> Result<[u8; 32]> {
    let bytes = bincode::serialize(data)?;
    Ok(hash_data(&bytes))
}

/// Validate an address format
pub fn validate_address(address: &Address) -> bool {
    // Basic validation - in production you'd check checksum, etc.
    !address.iter().all(|&b| b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_data() {
        let data = b"hello world";
        let hash1 = hash_data(data);
        let hash2 = hash_data(data);
        assert_eq!(hash1, hash2);
        
        let different_data = b"hello world!";
        let hash3 = hash_data(different_data);
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_validate_address() {
        let zero_address = [0u8; 20];
        assert!(!validate_address(&zero_address));
        
        let mut valid_address = [0u8; 20];
        valid_address[0] = 1;
        assert!(validate_address(&valid_address));
    }
}
