# Scripts Directory

Utility scripts for deploying and managing the u2048 application.

## Service Management

### `start_service.sh` - Standard Service
Start a standard Linera service for main chain or player chains.

```bash
# Usage: ./start_service.sh [PORT] [WALLET]

# Start on default port 8088
./scripts/start_service.sh

# Start on custom port
./scripts/start_service.sh 8090

# Start with custom wallet
./scripts/start_service.sh 8088 /path/to/wallet.json
```

**Use for:**
- Main chain
- Player chains
- Any chain that doesn't need batch message processing

---

### `start_leaderboard_service.sh` - Leaderboard Service
Start a Linera service optimized for leaderboard chains with batch message processing.

```bash
# Usage: ./start_leaderboard_service.sh [PORT] [BATCH_SIZE] [WALLET]

# Start with defaults (port 8089, 1000 messages/block)
./scripts/start_leaderboard_service.sh

# Start on custom port with 500 messages/block
./scripts/start_leaderboard_service.sh 8089 500

# Start with custom wallet
./scripts/start_leaderboard_service.sh 8089 1000 /path/to/wallet.json
```

**Configuration:**
- `--max-pending-message-bundles 1000` - Process up to 1000 messages per block
- `--listener-skip-process-inbox` - Queue messages until manual refresh

**Use for:**
- Leaderboard chains
- Any chain receiving high volumes of cross-chain messages

**Batch Size Recommendations:**
- **50-100**: Small tournaments (< 50 players)
- **200-500**: Medium tournaments (50-200 players)
- **1000+**: Large tournaments (200+ players) or stress testing

---

## Deployment Scripts

### `setup_caddy.sh`
Set up Caddy reverse proxy with automatic HTTPS.

```bash
./scripts/setup_caddy.sh yourdomain.com
```

**Features:**
- Automatic SSL certificate (Let's Encrypt)
- WebSocket support
- Firewall configuration
- Systemd service setup

---

### `update_domain.sh`
Update domain configuration across all config files.

```bash
./scripts/update_domain.sh yourdomain.com
```

**Updates:**
- Caddyfile
- Frontend environment files
- Configuration files

---

## Monitoring Scripts

### `monitor_service.sh`
Monitor Linera service and automatically handle application requests.

```bash
./scripts/monitor_service.sh
```

**Features:**
- Watches for `REQUEST_APPLICATION` events
- Auto-executes `linera request-application` commands
- Tracks processed requests to prevent duplicates

---

### `monitor_net_up.sh`
Monitor and maintain local Linera network.

```bash
./scripts/monitor_net_up.sh
```

---

## Example Multi-Service Deployment

Run multiple services for different chain types:

```bash
# Terminal 1: Main chain
./scripts/start_service.sh 8088

# Terminal 2: Leaderboard chain (high-volume batch processing)
./scripts/start_leaderboard_service.sh 8089 1000

# Terminal 3: Player chain
./scripts/start_service.sh 8090
```

---

## Environment Variables

All scripts support standard Linera environment variables:

```bash
# Set wallet location
export LINERA_WALLET=/path/to/wallet.json

# Set storage location
export LINERA_STORAGE=rocksdb:/path/to/storage

# Run service
./scripts/start_service.sh 8088
```

---

## Troubleshooting

### Service won't start
```bash
# Check if port is already in use
lsof -i :8088

# Check Linera installation
linera --version

# Check wallet exists
ls -la $LINERA_WALLET
```

### High message volumes causing issues
```bash
# Increase batch size for leaderboard chains
./scripts/start_leaderboard_service.sh 8089 2000

# Monitor message processing
# (check service logs for "Processed X messages")
```

### Permission denied
```bash
# Make scripts executable
chmod +x scripts/*.sh
```

---

## See Also

- [DEPLOYMENT.md](../DEPLOYMENT.md) - Full deployment guide
- [INBOX_BATCH_SIZE_FIX.md](../INBOX_BATCH_SIZE_FIX.md) - Batch processing details
- [Linera Documentation](https://linera.dev)
