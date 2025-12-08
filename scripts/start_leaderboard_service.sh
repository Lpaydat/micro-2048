#!/bin/bash

# Start Leaderboard Service with Batch Message Processing
# This script starts a Linera service optimized for leaderboard chains
# with high-volume message processing capabilities.

set -e

# Configuration
DEFAULT_PORT=8089
DEFAULT_BATCH_SIZE=1000
DEFAULT_WALLET=""

# Parse command line arguments
PORT="${1:-$DEFAULT_PORT}"
BATCH_SIZE="${2:-$DEFAULT_BATCH_SIZE}"
WALLET="${3:-$DEFAULT_WALLET}"

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Leaderboard Service Starter${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "${GREEN}Configuration:${NC}"
echo -e "  Port:                 ${YELLOW}${PORT}${NC}"
echo -e "  Batch Size:           ${YELLOW}${BATCH_SIZE}${NC} messages/block"
echo -e "  Skip Process Inbox:   ${YELLOW}enabled${NC}"

if [ -n "$WALLET" ]; then
    echo -e "  Wallet:               ${YELLOW}${WALLET}${NC}"
fi

echo ""
echo -e "${GREEN}Starting service...${NC}"
echo ""

# Build the command
CMD="linera --max-pending-message-bundles ${BATCH_SIZE}"

# Add wallet if specified
if [ -n "$WALLET" ]; then
    CMD="${CMD} --wallet ${WALLET}"
fi

# Add service subcommand with options
CMD="${CMD} service --port ${PORT} --listener-skip-process-inbox"

# Print the command
echo -e "${BLUE}Command:${NC}"
echo -e "${YELLOW}${CMD}${NC}"
echo ""

# Execute
exec $CMD
