// storage/scylla-adapter/src/dao.rs
use blockchain_core::{Address, TxHash, BlockHash, BlockHeight};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Account model for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountModel {
    pub address: Address,
    pub balance: u64,
    pub nonce: u64,
    pub last_updated: DateTime<Utc>,
    pub account_type: String, // "user" or "contract"
    pub code_hash: Option<BlockHash>, // For contract accounts
}

/// Transaction reference for address lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressTransaction {
    pub timestamp: DateTime<Utc>,
    pub tx_hash: TxHash,
    pub block_height: Option<BlockHeight>,
    pub tx_type: String,
    pub amount: u64,
    pub is_sender: bool,
}

/// Validation batch model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationBatch {
    pub queue_id: Uuid,
    pub batch_timestamp: DateTime<Utc>,
    pub tx_hashes: Vec<TxHash>,
    pub validation_status: ValidationStatus,
    pub validator_id: String,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub validation_result: Option<ValidationResult>,
}

/// Validation status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    Pending,
    Processing,
    Validated,
    Failed,
    Rejected,
}

impl std::fmt::Display for ValidationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationStatus::Pending => write!(f, "pending"),
            ValidationStatus::Processing => write!(f, "processing"),
            ValidationStatus::Validated => write!(f, "validated"),
            ValidationStatus::Failed => write!(f, "failed"),
            ValidationStatus::Rejected => write!(f, "rejected"),
        }
    }
}

impl std::str::FromStr for ValidationStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(ValidationStatus::Pending),
            "processing" => Ok(ValidationStatus::Processing),
            "validated" => Ok(ValidationStatus::Validated),
            "failed" => Ok(ValidationStatus::Failed),
            "rejected" => Ok(ValidationStatus::Rejected),
            _ => Err(format!("Invalid validation status: {}", s)),
        }
    }
}

/// Validation result details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub validated_transactions: Vec<TxHash>,
    pub failed_transactions: Vec<FailedTransaction>,
    pub gas_estimates: Vec<GasEstimate>,
    pub balance_changes: Vec<BalanceChange>,
    pub validation_time_ms: u64,
    pub error_message: Option<String>,
}

/// Failed transaction details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedTransaction {
    pub tx_hash: TxHash,
    pub error_code: String,
    pub error_message: String,
    pub suggested_gas_limit: Option<u64>,
}

/// Gas estimation for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    pub tx_hash: TxHash,
    pub estimated_gas: u64,
    pub gas_price_suggestion: u64,
    pub execution_time_estimate_ms: u64,
}

/// Balance change tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub address: Address,
    pub old_balance: u64,
    pub new_balance: u64,
    pub old_nonce: u64,
    pub new_nonce: u64,
}

/// Relayer batch model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayerBatch {
    pub commitment_id: Uuid,
    pub batch_timestamp: DateTime<Utc>,
    pub tx_hashes: Vec<TxHash>,
    pub status: RelayerStatus,
    pub relayer_id: String,
    pub retry_count: u32,
    pub last_attempt: Option<DateTime<Utc>>,
    pub target_block_height: Option<BlockHeight>,
    pub commitment_data: Option<CommitmentData>,
}

/// Relayer status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelayerStatus {
    Queued,
    Processing,
    Committed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for RelayerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayerStatus::Queued => write!(f, "queued"),
            RelayerStatus::Processing => write!(f, "processing"),
            RelayerStatus::Committed => write!(f, "committed"),
            RelayerStatus::Failed => write!(f, "failed"),
            RelayerStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for RelayerStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "queued" => Ok(RelayerStatus::Queued),
            "processing" => Ok(RelayerStatus::Processing),
            "committed" => Ok(RelayerStatus::Committed),
            "failed" => Ok(RelayerStatus::Failed),
            "cancelled" => Ok(RelayerStatus::Cancelled),
            _ => Err(format!("Invalid relayer status: {}", s)),
        }
    }
}

/// Commitment data for relayer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentData {
    pub merkle_root: BlockHash,
    pub transaction_count: u32,
    pub total_gas_used: u64,
    pub total_fees: u64,
    pub batch_hash: BlockHash,
    pub proof_data: Vec<u8>, // Cryptographic proof
}

/// Network peer model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPeer {
    pub peer_id: String,
    pub ip_address: std::net::IpAddr,
    pub port: u16,
    pub last_seen: DateTime<Utc>,
    pub version: String,
    pub chain_height: BlockHeight,
    pub status: PeerStatus,
    pub connection_count: u32,
}

/// Peer status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PeerStatus {
    Connected,
    Disconnected,
    Banned,
    Syncing,
}

impl std::fmt::Display for PeerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PeerStatus::Connected => write!(f, "connected"),
            PeerStatus::Disconnected => write!(f, "disconnected"),
            PeerStatus::Banned => write!(f, "banned"),
            PeerStatus::Syncing => write!(f, "syncing"),
        }
    }
}

impl std::str::FromStr for PeerStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "connected" => Ok(PeerStatus::Connected),
            "disconnected" => Ok(PeerStatus::Disconnected),
            "banned" => Ok(PeerStatus::Banned),
            "syncing" => Ok(PeerStatus::Syncing),
            _ => Err(format!("Invalid peer status: {}", s)),
        }
    }
}

/// Chain statistics model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub latest_block_height: BlockHeight,
    pub avg_block_time: f64, // in seconds
    pub network_hash_rate: u64,
    pub active_addresses: u64,
}

/// Hourly chain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyChainStats {
    pub stat_date: chrono::NaiveDate,
    pub stat_hour: u8,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub total_value: u64,
    pub total_fees: u64,
    pub avg_block_time: f64,
    pub avg_tx_per_block: f64,
    pub network_hash_rate: u64,
    pub active_addresses: u64,
}

/// System configuration model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub config_key: String,
    pub config_value: String,
    pub updated_at: DateTime<Utc>,
    pub updated_by: String,
}

/// Pending transaction priority model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransactionPriority {
    pub tx_hash: TxHash,
    pub priority_score: u64, // gas_price * gas_limit
    pub timestamp: DateTime<Utc>,
    pub sender: Address,
    pub nonce: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
}

/// Transaction volume statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionVolumeStats {
    pub hour: DateTime<Utc>,
    pub transaction_count: u64,
    pub total_volume: u64,
    pub avg_transaction_size: f64,
    pub unique_addresses: u64,
}

/// Address activity statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressActivityStats {
    pub address: Address,
    pub transaction_count: u64,
    pub total_sent: u64,
    pub total_received: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub is_contract: bool,
}

/// Block production statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProductionStats {
    pub hour: DateTime<Utc>,
    pub blocks_produced: u64,
    pub avg_block_time: f64,
    pub min_block_time: f64,
    pub max_block_time: f64,
    pub total_transactions: u64,
    pub avg_tx_per_block: f64,
}

/// Mempool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStats {
    pub pending_count: u64,
    pub total_pending_value: u64,
    pub avg_gas_price: f64,
    pub min_gas_price: u64,
    pub max_gas_price: u64,
    pub oldest_pending_age_seconds: u64,
    pub newest_pending_age_seconds: u64,
}

impl ValidationBatch {
    pub fn new(tx_hashes: Vec<TxHash>, validator_id: String) -> Self {
        Self {
            queue_id: Uuid::new_v4(),
            batch_timestamp: Utc::now(),
            tx_hashes,
            validation_status: ValidationStatus::Pending,
            validator_id,
            started_at: None,
            completed_at: None,
            validation_result: None,
        }
    }

    pub fn start_processing(&mut self) {
        self.validation_status = ValidationStatus::Processing;
        self.started_at = Some(Utc::now());
    }

    pub fn complete_validation(&mut self, result: ValidationResult) {
        self.validation_status = if result.is_valid {
            ValidationStatus::Validated
        } else {
            ValidationStatus::Failed
        };
        self.completed_at = Some(Utc::now());
        self.validation_result = Some(result);
    }
}

impl RelayerBatch {
    pub fn new(tx_hashes: Vec<TxHash>, relayer_id: String) -> Self {
        Self {
            commitment_id: Uuid::new_v4(),
            batch_timestamp: Utc::now(),
            tx_hashes,
            status: RelayerStatus::Queued,
            relayer_id,
            retry_count: 0,
            last_attempt: None,
            target_block_height: None,
            commitment_data: None,
        }
    }

    pub fn start_processing(&mut self, target_block_height: BlockHeight) {
        self.status = RelayerStatus::Processing;
        self.last_attempt = Some(Utc::now());
        self.target_block_height = Some(target_block_height);
    }

    pub fn mark_committed(&mut self, commitment_data: CommitmentData) {
        self.status = RelayerStatus::Committed;
        self.commitment_data = Some(commitment_data);
    }

    pub fn mark_failed(&mut self) {
        self.status = RelayerStatus::Failed;
        self.retry_count += 1;
        self.last_attempt = Some(Utc::now());
    }

    pub fn can_retry(&self, max_retries: u32) -> bool {
        self.retry_count < max_retries && self.status == RelayerStatus::Failed
    }
}

impl NetworkPeer {
    pub fn new(
        peer_id: String,
        ip_address: std::net::IpAddr,
        port: u16,
        version: String,
    ) -> Self {
        Self {
            peer_id,
            ip_address,
            port,
            last_seen: Utc::now(),
            version,
            chain_height: 0,
            status: PeerStatus::Disconnected,
            connection_count: 0,
        }
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }

    pub fn connect(&mut self, chain_height: BlockHeight) {
        self.status = PeerStatus::Connected;
        self.chain_height = chain_height;
        self.connection_count += 1;
        self.update_last_seen();
    }

    pub fn disconnect(&mut self) {
        self.status = PeerStatus::Disconnected;
        self.update_last_seen();
    }

    pub fn ban(&mut self) {
        self.status = PeerStatus::Banned;
        self.update_last_seen();
    }

    pub fn is_stale(&self, stale_threshold_seconds: i64) -> bool {
        let threshold = Utc::now() - chrono::Duration::seconds(stale_threshold_seconds);
        self.last_seen < threshold
    }
}