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
