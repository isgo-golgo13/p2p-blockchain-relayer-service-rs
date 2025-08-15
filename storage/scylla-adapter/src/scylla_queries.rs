// storage/scylla-adapter/src/queries.rs

// Block operations
pub const INSERT_BLOCK: &str = r#"
    INSERT INTO blocks (
        height, hash, previous_hash, merkle_root, timestamp, nonce, 
        difficulty, version, transaction_count, size, total_value, 
        total_fees, block_data
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#;

pub const GET_BLOCK_BY_HEIGHT: &str = r#"
    SELECT height, hash, previous_hash, merkle_root, timestamp, nonce,
           difficulty, version, transaction_count, size, total_value,
           total_fees, block_data
    FROM blocks WHERE height = ?
"#;

pub const GET_BLOCK_BY_HASH: &str = r#"
    SELECT height FROM blocks_by_hash WHERE hash = ?
"#;

pub const GET_RECENT_BLOCKS: &str = r#"
    SELECT height, hash, timestamp, transaction_count, total_value, total_fees
    FROM recent_blocks 
    ORDER BY timestamp DESC 
    LIMIT ?
"#;

// Transaction operations
pub const INSERT_TRANSACTION: &str = r#"
    INSERT INTO transactions (
        tx_hash, block_height, tx_index, sender, recipient, amount,
        tx_type, nonce, gas_limit, gas_price, timestamp, status,
        signature, tx_data
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#;

pub const GET_TRANSACTION: &str = r#"
    SELECT tx_hash, block_height, tx_index, sender, recipient, amount,
           tx_type, nonce, gas_limit, gas_price, timestamp, status,
           signature, tx_data
    FROM transactions WHERE tx_hash = ?
"#;

pub const INSERT_TX_BY_ADDRESS: &str = r#"
    INSERT INTO transactions_by_address (
        address, timestamp, tx_hash, block_height, tx_type, amount, is_sender
    ) VALUES (?, ?, ?, ?, ?, ?, ?)
"#;

pub const GET_TX_BY_ADDRESS: &str = r#"
    SELECT timestamp, tx_hash, block_height, tx_type, amount, is_sender
    FROM transactions_by_address 
    WHERE address = ? 
    ORDER BY timestamp DESC 
    LIMIT ?
"#;

pub const GET_TX_BY_BLOCK: &str = r#"
    SELECT tx_hash, timestamp
    FROM transactions_by_block 
    WHERE block_height = ? 
    ORDER BY tx_index ASC
"#;

// Pending transaction operations
pub const INSERT_PENDING_TX: &str = r#"
    INSERT INTO pending_transactions (
        tx_hash, priority_score, timestamp, sender, nonce,
        gas_price, gas_limit, tx_data
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#;

pub const DELETE_PENDING_TX: &str = r#"
    DELETE FROM pending_transactions 
    WHERE priority_score = ? AND timestamp = ? AND tx_hash = ?
"#;

pub const GET_PENDING_TX_BY_PRIORITY: &str = r#"
    SELECT tx_data 
    FROM pending_transactions 
    ORDER BY priority_score DESC, timestamp ASC 
    LIMIT ?
"#;

pub const GET_PENDING_TX_BY_SENDER: &str = r#"
    SELECT tx_hash, nonce, tx_data
    FROM pending_transactions 
    WHERE sender = ? 
    ALLOW FILTERING
"#;

// Account operations
pub const UPDATE_ACCOUNT: &str = r#"
    INSERT INTO accounts (address, balance, nonce, last_updated, account_type, code_hash)
    VALUES (?, ?, ?, ?, ?, ?)
"#;

pub const GET_ACCOUNT: &str = r#"
    SELECT address, balance, nonce, last_updated, account_type, code_hash
    FROM accounts WHERE address = ?
"#;

pub const GET_ACCOUNT_BALANCE: &str = r#"
    SELECT balance FROM accounts WHERE address = ?
"#;

pub const GET_ACCOUNT_NONCE: &str = r#"
    SELECT nonce FROM accounts WHERE address = ?
"#;

// Validation queue operations
pub const INSERT_VALIDATION_BATCH: &str = r#"
    INSERT INTO validation_queue (
        queue_id, batch_timestamp, tx_hashes, validation_status,
        validator_id, started_at, validation_result
    ) VALUES (?, ?, ?, ?, ?, ?, ?)
"#;

pub const UPDATE_VALIDATION_STATUS: &str = r#"
    UPDATE validation_queue 
    SET validation_status = ?, completed_at = ?, validation_result = ?
    WHERE batch_timestamp = ? AND queue_id = ?
"#;

pub const GET_PENDING_VALIDATION: &str = r#"
    SELECT queue_id, batch_timestamp, tx_hashes, validator_id, started_at
    FROM validation_queue 
    WHERE validation_status = 'pending'
    LIMIT ?
"#;

pub const GET_VALIDATION_RESULT: &str = r#"
    SELECT validation_status, validation_result, completed_at
    FROM validation_queue 
    WHERE batch_timestamp = ? AND queue_id = ?
"#;

// Relayer queue operations
pub const INSERT_RELAYER_BATCH: &str = r#"
    INSERT INTO relayer_queue (
        commitment_id, batch_timestamp, tx_hashes, status,
        relayer_id, retry_count, last_attempt, commitment_data
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#;

pub const UPDATE_RELAYER_STATUS: &str = r#"
    UPDATE relayer_queue 
    SET status = ?, retry_count = ?, last_attempt = ?, target_block_height = ?
    WHERE batch_timestamp = ? AND commitment_id = ?
"#;

pub const GET_PENDING_RELAYER_BATCHES: &str = r#"
    SELECT commitment_id, batch_timestamp, tx_hashes, relayer_id, 
           retry_count, commitment_data
    FROM relayer_queue 
    WHERE status = 'queued'
    LIMIT ?
"#;

pub const GET_FAILED_RELAYER_BATCHES: &str = r#"
    SELECT commitment_id, batch_timestamp, tx_hashes, retry_count
    FROM relayer_queue 
    WHERE status = 'failed' AND retry_count < ?
    LIMIT ?
"#;

// Network peer operations
pub const UPDATE_PEER: &str = r#"
    INSERT INTO network_peers (
        peer_id, ip_address, port, last_seen, version,
        chain_height, status, connection_count
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
"#;

pub const GET_ACTIVE_PEERS: &str = r#"
    SELECT peer_id, ip_address, port, last_seen, version, chain_height
    FROM network_peers 
    WHERE status = 'connected'
    LIMIT ?
"#;

pub const GET_PEER_BY_ID: &str = r#"
    SELECT peer_id, ip_address, port, last_seen, version, 
           chain_height, status, connection_count
    FROM network_peers 
    WHERE peer_id = ?
"#;

// Chain statistics operations
pub const INSERT_CHAIN_STATS: &str = r#"
    INSERT INTO chain_stats (
        stat_date, stat_hour, total_blocks, total_transactions,
        total_value, total_fees, avg_block_time, avg_tx_per_block,
        network_hash_rate, active_addresses
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
"#;

pub const GET_CHAIN_STATS_BY_DATE: &str = r#"
    SELECT stat_hour, total_blocks, total_transactions, total_value,
           total_fees, avg_block_time, avg_tx_per_block, network_hash_rate,
           active_addresses
    FROM chain_stats 
    WHERE stat_date = ?
    ORDER BY stat_hour DESC
"#;

pub const GET_LATEST_CHAIN_STATS: &str = r#"
    SELECT total_blocks, total_transactions, total_value, total_fees,
           avg_block_time, avg_tx_per_block, network_hash_rate, active_addresses
    FROM chain_stats 
    ORDER BY stat_date DESC, stat_hour DESC 
    LIMIT 1
"#;

// System configuration operations
pub const GET_CONFIG: &str = r#"
    SELECT config_value FROM system_config WHERE config_key = ?
"#;

pub const SET_CONFIG: &str = r#"
    INSERT INTO system_config (config_key, config_value, updated_at, updated_by)
    VALUES (?, ?, ?, ?)
"#;

pub const GET_ALL_CONFIG: &str = r#"
    SELECT config_key, config_value, updated_at, updated_by
    FROM system_config
"#;

// Advanced query patterns
pub const GET_TRANSACTION_VOLUME_BY_HOUR: &str = r#"
    SELECT DATE_FORMAT(timestamp, '%Y-%m-%d %H:00:00') AS hour,
           COUNT(*) AS tx_count,
           SUM(amount) AS total_volume
    FROM transactions 
    WHERE timestamp >= ? AND timestamp < ?
    GROUP BY hour
    ORDER BY hour ASC
"#;

pub const GET_TOP_ADDRESSES_BY_TRANSACTION_COUNT: &str = r#"
    SELECT address, COUNT(*) AS tx_count
    FROM transactions_by_address 
    WHERE timestamp >= ? AND timestamp < ?
    GROUP BY address
    ORDER BY tx_count DESC
    LIMIT ?
"#;

pub const GET_BLOCK_PRODUCTION_RATE: &str = r#"
    SELECT DATE_FORMAT(timestamp, '%Y-%m-%d %H:00:00') AS hour,
           COUNT(*) AS blocks_produced,
           AVG(EXTRACT(EPOCH FROM (timestamp - LAG(timestamp) OVER (ORDER BY height)))) AS avg_block_time
    FROM blocks 
    WHERE timestamp >= ? AND timestamp < ?
    GROUP BY hour
    ORDER BY hour ASC
"#;

// Cleanup operations
pub const CLEANUP_OLD_PENDING_TX: &str = r#"
    DELETE FROM pending_transactions 
    WHERE timestamp < ?
"#;

pub const CLEANUP_OLD_VALIDATION_QUEUE: &str = r#"
    DELETE FROM validation_queue 
    WHERE batch_timestamp < ?
"#;

pub const CLEANUP_OLD_RELAYER_QUEUE: &str = r#"
    DELETE FROM relayer_queue 
    WHERE batch_timestamp < ? AND status IN ('committed', 'failed')
"#;

pub const CLEANUP_OLD_PEER_DATA: &str = r#"
    DELETE FROM network_peers 
    WHERE last_seen < ?
"#;
