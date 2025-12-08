#!/bin/bash

# Start Standard Linera Service
# For main chain and player chains (no special configuration)

set -e

# Configuration
DEFAULT_PORT=8088
DEFAULT_WALLET=""

# Parse command line arguments
PORT="${1:-$DEFAULT_PORT}"
WALLET="${2:-$DEFAULT_WALLET}"

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Standard Service Starter${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "${GREEN}Configuration:${NC}"
echo -e "  Port:                 ${YELLOW}${PORT}${NC}"

if [ -n "$WALLET" ]; then
    echo -e "  Wallet:               ${YELLOW}${WALLET}${NC}"
fi

echo ""
echo -e "${GREEN}Starting service...${NC}"
echo ""

# Build the command
CMD="linera"

# Add wallet if specified
if [ -n "$WALLET" ]; then
    CMD="${CMD} --wallet ${WALLET}"
fi

# Add service subcommand with options
CMD="${CMD} service --port ${PORT}"

# Print the command
echo -e "${BLUE}Command:${NC}"
echo -e "${YELLOW}${CMD}${NC}"
echo ""

# Execute
exec $CMD
