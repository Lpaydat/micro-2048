# Suggested Commands for GameHub Development

## Build Commands
```bash
# Build for WASM target (required for Linera)
cargo build --release --target wasm32-unknown-unknown

# Regular build
cargo build
```

## Test Commands
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Test with environment variables
export LINERA_WALLET="/tmp/.tmpXXXXXX/wallet_0.json"
export LINERA_STORAGE="rocksdb:/tmp/.tmpXXXXXX/client_0.db"
cargo test
```

## Linera Deployment Commands
```bash
# Publish and create application
linera publish-and-create <contract_bytecode> <service_bytecode> --json-argument '<init_args>'
```

## Development Commands
```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check code
cargo check
```

## Binaries
- `gamehub_contract`: Contract entry point
- `gamehub_service`: Service entry point