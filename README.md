# P2P Blockchain Service w/ Off-Chain/On-Chain Relayer (Rust)
P2P Blockchain Service w/ Off-Chain-to-On-Chain Relayer Service using Rust, Rust Dioxus WASM Web Framework, Rust Async Tokio, Rust LibP2P, React.js and D3.js Transactions Visualization.


## Create Project Structure

```shell
# Create the workspace structure
mkdir -p blockchain/{blockchain-core,consensus,crypto}
mkdir -p storage/{scylla-adapter,storage-traits}
mkdir -p validation/{on-chain-validator,off-chain-validator,validation-core}
mkdir -p relayer/{relayer-server,relayer-api,gateway-service}
mkdir -p p2p/{p2p-network,rpc-server}
mkdir -p frontend/{web-ui,dioxus-admin}
mkdir -p tools/{cli-tools,dev-tools}


# Initilaize the workspace project crates
cargo init blockchain/blockchain-core --lib
cargo init blockchain/consensus --lib
cargo init blockchain/crypto --lib
cargo init storage/scylla-adapter --lib
cargo init storage/storage-traits --lib
cargo init validation/on-chain-validator --lib
cargo init validation/off-chain-validator --lib
cargo init validation/validation-core --lib
cargo init relayer/relayer-server --bin
cargo init relayer/relayer-api --lib
cargo init relayer/gateway-core --lib
cargo init p2p/p2p-network --lib
cargo init p2p/rpc-server --bin
cargo init frontend/dioxus-admin --bin
cargo init tools/cli-tools --bin
cargo init tools/dev-tools --lib
```


## Create Transaction Database  (ScyllaDB Schema)

```cql
-- ScyllaDB Schema for Blockchain Service
-- This file should be run to create the keyspace and tables

-- Create keyspace with replication strategy
CREATE KEYSPACE IF NOT EXISTS blockchain 
WITH replication = {
    'class': 'NetworkTopologyStrategy',
    'datacenter1': 3
} AND durable_writes = true;

USE blockchain;

-- Blocks table - main chain storage
CREATE TABLE IF NOT EXISTS blocks (
    height bigint,
    hash blob,
    previous_hash blob,
    merkle_root blob,
    timestamp timestamp,
    nonce bigint,
    difficulty int,
    version int,
    transaction_count int,
    size bigint,
    total_value bigint,
    total_fees bigint,
    block_data blob, -- Serialized complete block
    PRIMARY KEY (height)
) WITH CLUSTERING ORDER BY (height DESC)
  AND comment = 'Main blockchain blocks storage'
  AND gc_grace_seconds = 864000
  AND compaction = {
    'class': 'LeveledCompactionStrategy',
    'sstable_size_in_mb': 160
  };

-- Block hash index for quick lookups
CREATE TABLE IF NOT EXISTS blocks_by_hash (
    hash blob,
    height bigint,
    PRIMARY KEY (hash)
) WITH comment = 'Block lookup by hash';

-- Transactions table - all transactions
CREATE TABLE IF NOT EXISTS transactions (
    tx_hash blob,
    block_height bigint,
    tx_index int,
    sender blob,
    recipient blob,
    amount bigint,
    tx_type text,
    nonce bigint,
    gas_limit bigint,
    gas_price bigint,
    timestamp timestamp,
    status text,
    signature blob,
    tx_data blob, -- Serialized complete transaction
    PRIMARY KEY (tx_hash)
) WITH comment = 'All blockchain transactions'
  AND gc_grace_seconds = 864000;

-- Transactions by block - for block transaction lookups
CREATE TABLE IF NOT EXISTS transactions_by_block (
    block_height bigint,
    tx_index int,
    tx_hash blob,
    timestamp timestamp,
    PRIMARY KEY (block_height, tx_index)
) WITH CLUSTERING ORDER BY (tx_index ASC)
  AND comment = 'Transactions grouped by block';

-- Transactions by address - for account history
CREATE TABLE IF NOT EXISTS transactions_by_address (
    address blob,
    timestamp timestamp,
    tx_hash blob,
    block_height bigint,
    tx_type text,
    amount bigint,
    is_sender boolean,
    PRIMARY KEY (address, timestamp, tx_hash)
) WITH CLUSTERING ORDER BY (timestamp DESC, tx_hash ASC)
  AND comment = 'Transaction history by address'
  AND gc_grace_seconds = 864000;

-- Pending transactions (mempool)
CREATE TABLE IF NOT EXISTS pending_transactions (
    tx_hash blob,
    priority_score bigint, -- gas_price * gas_limit for ordering
    timestamp timestamp,
    sender blob,
    nonce bigint,
    gas_price bigint,
    gas_limit bigint,
    tx_data blob, -- Serialized transaction
    PRIMARY KEY (priority_score, timestamp, tx_hash)
) WITH CLUSTERING ORDER BY (timestamp ASC, tx_hash ASC)
  AND comment = 'Pending transactions in mempool'
  AND default_time_to_live = 3600; -- Auto-expire after 1 hour

-- Account balances and nonces
CREATE TABLE IF NOT EXISTS accounts (
    address blob,
    balance bigint,
    nonce bigint,
    last_updated timestamp,
    account_type text, -- 'user' or 'contract'
    code_hash blob, -- For contract accounts
    PRIMARY KEY (address)
) WITH comment = 'Account states and balances';

-- Off-chain validation queue
CREATE TABLE IF NOT EXISTS validation_queue (
    queue_id uuid,
    batch_timestamp timestamp,
    tx_hashes list<blob>,
    validation_status text, -- 'pending', 'validated', 'failed'
    validator_id text,
    started_at timestamp,
    completed_at timestamp,
    validation_result blob, -- Serialized validation result
    PRIMARY KEY (batch_timestamp, queue_id)
) WITH CLUSTERING ORDER BY (queue_id ASC)
  AND comment = 'Off-chain validation processing queue'
  AND default_time_to_live = 86400; -- 24 hours

-- Relayer commitment queue
CREATE TABLE IF NOT EXISTS relayer_queue (
    commitment_id uuid,
    batch_timestamp timestamp,
    tx_hashes list<blob>,
    status text, -- 'queued', 'processing', 'committed', 'failed'
    relayer_id text,
    retry_count int,
    last_attempt timestamp,
    target_block_height bigint,
    commitment_data blob, -- Serialized batch data
    PRIMARY KEY (batch_timestamp, commitment_id)
) WITH CLUSTERING ORDER BY (commitment_id ASC)
  AND comment = 'Relayer commitment processing queue'
  AND default_time_to_live = 86400; -- 24 hours

-- Network peers and P2P state
CREATE TABLE IF NOT EXISTS network_peers (
    peer_id text,
    ip_address inet,
    port int,
    last_seen timestamp,
    version text,
    chain_height bigint,
    status text, -- 'connected', 'disconnected', 'banned'
    connection_count int,
    PRIMARY KEY (peer_id)
) WITH comment = 'P2P network peer information'
  AND default_time_to_live = 7200; -- 2 hours

-- Chain statistics and metrics
CREATE TABLE IF NOT EXISTS chain_stats (
    stat_date date,
    stat_hour int,
    total_blocks bigint,
    total_transactions bigint,
    total_value bigint,
    total_fees bigint,
    avg_block_time double,
    avg_tx_per_block double,
    network_hash_rate bigint,
    active_addresses bigint,
    PRIMARY KEY (stat_date, stat_hour)
) WITH CLUSTERING ORDER BY (stat_hour DESC)
  AND comment = 'Blockchain statistics by hour'
  AND gc_grace_seconds = 2592000; -- 30 days

-- System configuration and state
CREATE TABLE IF NOT EXISTS system_config (
    config_key text,
    config_value text,
    updated_at timestamp,
    updated_by text,
    PRIMARY KEY (config_key)
) WITH comment = 'System configuration parameters';

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS tx_sender_idx ON transactions (sender);
CREATE INDEX IF NOT EXISTS tx_recipient_idx ON transactions (recipient);
CREATE INDEX IF NOT EXISTS tx_timestamp_idx ON transactions (timestamp);
CREATE INDEX IF NOT EXISTS tx_status_idx ON transactions (status);

CREATE INDEX IF NOT EXISTS pending_tx_sender_idx ON pending_transactions (sender);
CREATE INDEX IF NOT EXISTS pending_tx_nonce_idx ON pending_transactions (nonce);

CREATE INDEX IF NOT EXISTS validation_status_idx ON validation_queue (validation_status);
CREATE INDEX IF NOT EXISTS relayer_status_idx ON relayer_queue (status);

-- Insert initial system configuration
INSERT INTO system_config (config_key, config_value, updated_at, updated_by) 
VALUES ('chain_id', '1', toTimestamp(now()), 'system');

INSERT INTO system_config (config_key, config_value, updated_at, updated_by) 
VALUES ('block_time_target', '12', toTimestamp(now()), 'system');

INSERT INTO system_config (config_key, config_value, updated_at, updated_by) 
VALUES ('max_block_size', '1048576', toTimestamp(now()), 'system');

INSERT INTO system_config (config_key, config_value, updated_at, updated_by) 
VALUES ('min_gas_price', '1000000000', toTimestamp(now()), 'system');

-- Create materialized views for common queries
CREATE MATERIALIZED VIEW IF NOT EXISTS recent_blocks AS
    SELECT height, hash, timestamp, transaction_count, total_value, total_fees
    FROM blocks
    WHERE height IS NOT NULL AND hash IS NOT NULL
    PRIMARY KEY (timestamp, height)
    WITH CLUSTERING ORDER BY (height DESC);

CREATE MATERIALIZED VIEW IF NOT EXISTS recent_transactions AS
    SELECT tx_hash, timestamp, sender, recipient, amount, status
    FROM transactions
    WHERE tx_hash IS NOT NULL AND timestamp IS NOT NULL
    PRIMARY KEY (timestamp, tx_hash)
    WITH CLUSTERING ORDER BY (tx_hash ASC);

-- Performance optimization hints
-- For time-series queries, consider using time-bucketing
-- For high-write workloads, tune compaction strategy
-- Monitor partition sizes and consider splitting large partitions
-- Use prepared statements for better performance
-- Consider using user-defined types (UDT) for complex nested data
```


## Create Transaction Visualization Dash

```shell
# Step 12: Create React + D3.js Frontend Project

# Navigate to the frontend directory
cd frontend/tx-ui-dash

# Create React application with JavaScript
npm create vite@latest . -- --template react

# Install D3.js and related dependencies
npm install d3

# Install additional UI and utility libraries
npm install axios
npm install @emotion/react @emotion/styled
npm install @mui/material @mui/icons-material
npm install react-router-dom
npm install recharts
npm install react-query
npm install date-fns
npm install lodash

# Install development dependencies
npm install --save-dev eslint-plugin-react-hooks
npm install --save-dev prettier

# Create directory structure
mkdir -p src/components/{BlockVisualization,TransactionList,Dashboard,Layout}
mkdir -p src/services
mkdir -p src/types
mkdir -p src/hooks
mkdir -p src/utils
mkdir -p src/styles
mkdir -p public/assets

# Create environment configuration
echo "REACT_APP_API_BASE_URL=http://localhost:8080" > .env.local
echo "REACT_APP_WEBSOCKET_URL=ws://localhost:8081" >> .env.local
echo "REACT_APP_BLOCKCHAIN_NAME=RolexChain" >> .env.local

echo "React project initialized with D3.js and JavaScript"
```