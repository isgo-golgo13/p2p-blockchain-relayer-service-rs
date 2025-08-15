// storage/scylla-adapter/src/scylla-config.rs
use serde::{Deserialize, Serialize};

/// ScyllaDB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScyllaConfig {
    /// List of ScyllaDB node addresses
    pub nodes: Vec<String>,
    /// Keyspace name
    pub keyspace: String,
    /// Username for authentication
    pub username: String,
    /// Password for authentication
    pub password: String,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Request timeout in milliseconds
    pub request_timeout_ms: u64,
    /// Maximum number of connections per node
    pub max_connections_per_node: usize,
    /// Connection pool size
    pub pool_size: usize,
    /// Whether to use compression
    pub use_compression: bool,
    /// Consistency level for reads
    pub read_consistency: String,
    /// Consistency level for writes
    pub write_consistency: String,
    /// Retry policy configuration
    pub retry_policy: RetryPolicyConfig,
    /// Load balancing policy
    pub load_balancing_policy: String,
}

/// Retry policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicyConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
}

impl Default for ScyllaConfig {
    fn default() -> Self {
        Self {
            nodes: vec!["127.0.0.1:9042".to_string()],
            keyspace: "blockchain".to_string(),
            username: "cassandra".to_string(),
            password: "cassandra".to_string(),
            connection_timeout_ms: 5000,
            request_timeout_ms: 10000,
            max_connections_per_node: 10,
            pool_size: 20,
            use_compression: true,
            read_consistency: "LOCAL_QUORUM".to_string(),
            write_consistency: "LOCAL_QUORUM".to_string(),
            retry_policy: RetryPolicyConfig::default(),
            load_balancing_policy: "DcAwareRoundRobinPolicy".to_string(),
        }
    }
}

impl Default for RetryPolicyConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
            exponential_backoff: true,
        }
    }
}

impl ScyllaConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, std::env::VarError> {
        let mut config = Self::default();
        
        if let Ok(nodes) = std::env::var("SCYLLA_NODES") {
            config.nodes = nodes.split(',').map(|s| s.trim().to_string()).collect();
        }
        
        if let Ok(keyspace) = std::env::var("SCYLLA_KEYSPACE") {
            config.keyspace = keyspace;
        }
        
        if let Ok(username) = std::env::var("SCYLLA_USERNAME") {
            config.username = username;
        }
        
        if let Ok(password) = std::env::var("SCYLLA_PASSWORD") {
            config.password = password;
        }
        
        if let Ok(timeout) = std::env::var("SCYLLA_CONNECTION_TIMEOUT_MS") {
            config.connection_timeout_ms = timeout.parse().unwrap_or(config.connection_timeout_ms);
        }
        
        if let Ok(timeout) = std::env::var("SCYLLA_REQUEST_TIMEOUT_MS") {
            config.request_timeout_ms = timeout.parse().unwrap_or(config.request_timeout_ms);
        }
        
        if let Ok(max_conn) = std::env::var("SCYLLA_MAX_CONNECTIONS_PER_NODE") {
            config.max_connections_per_node = max_conn.parse().unwrap_or(config.max_connections_per_node);
        }
        
        if let Ok(pool_size) = std::env::var("SCYLLA_POOL_SIZE") {
            config.pool_size = pool_size.parse().unwrap_or(config.pool_size);
        }
        
        if let Ok(compression) = std::env::var("SCYLLA_USE_COMPRESSION") {
            config.use_compression = compression.parse().unwrap_or(config.use_compression);
        }
        
        if let Ok(consistency) = std::env::var("SCYLLA_READ_CONSISTENCY") {
            config.read_consistency = consistency;
        }
        
        if let Ok(consistency) = std::env::var("SCYLLA_WRITE_CONSISTENCY") {
            config.write_consistency = consistency;
        }
        
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.nodes.is_empty() {
            return Err("At least one ScyllaDB node must be specified".to_string());
        }
        
        if self.keyspace.is_empty() {
            return Err("Keyspace name cannot be empty".to_string());
        }
        
        if self.username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }
        
        if self.connection_timeout_ms == 0 {
            return Err("Connection timeout must be greater than 0".to_string());
        }
        
        if self.request_timeout_ms == 0 {
            return Err("Request timeout must be greater than 0".to_string());
        }
        
        if self.max_connections_per_node == 0 {
            return Err("Max connections per node must be greater than 0".to_string());
        }
        
        if self.pool_size == 0 {
            return Err("Pool size must be greater than 0".to_string());
        }
        
        // Validate consistency levels
        let valid_consistency = [
            "ANY", "ONE", "TWO", "THREE", "QUORUM", "ALL",
            "LOCAL_QUORUM", "EACH_QUORUM", "SERIAL", "LOCAL_SERIAL", "LOCAL_ONE"
        ];
        
        if !valid_consistency.contains(&self.read_consistency.as_str()) {
            return Err(format!("Invalid read consistency level: {}", self.read_consistency));
        }
        
        if !valid_consistency.contains(&self.write_consistency.as_str()) {
            return Err(format!("Invalid write consistency level: {}", self.write_consistency));
        }
        
        Ok(())
    }
}
