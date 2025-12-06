#!/bin/bash

# 🎯 Leaderboard Stress Test Runner
# Easy wrapper script for running leaderboard stress tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Display banner
echo -e "${BLUE}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║         🎯 Leaderboard Stress Test Runner 🎯              ║"
echo "╔════════════════════════════════════════════════════════════╗"
echo -e "${NC}"

# Check if k6 is installed
if ! command -v k6 &> /dev/null; then
    echo -e "${RED}❌ Error: k6 is not installed${NC}"
    echo "Install it from: https://k6.io/docs/getting-started/installation/"
    echo "  macOS: brew install k6"
    echo "  Linux: sudo apt install k6"
    exit 1
fi

# Check if tournament ID is provided
if [ -z "$1" ]; then
    echo -e "${RED}❌ Error: Tournament ID is required${NC}"
    echo ""
    echo "Usage: $0 <TOURNAMENT_ID> [SCENARIO]"
    echo ""
    echo "Available scenarios:"
    echo "  light    - 10 players, slow gameplay (default)"
    echo "  medium   - 30 players, moderate speed"
    echo "  heavy    - 50 players, fast gameplay"
    echo "  stress   - 100 players, maximum stress"
    echo "  custom   - Use environment variables for full control"
    echo ""
    echo "Example:"
    echo "  $0 abc123def456 light"
    echo "  $0 abc123def456 stress"
    echo ""
    exit 1
fi

TOURNAMENT_ID=$1
SCENARIO=${2:-light}

# Default to local environment unless ENVIRONMENT is set
ENVIRONMENT=${ENVIRONMENT:-local}

echo -e "${GREEN}🎯 Tournament ID: $TOURNAMENT_ID${NC}"
echo -e "${GREEN}📊 Scenario: $SCENARIO${NC}"
echo -e "${GREEN}🌍 Environment: $ENVIRONMENT${NC}"
echo ""

# Scenario configurations
case $SCENARIO in
    light)
        echo -e "${BLUE}Light Load Configuration:${NC}"
        echo "  • 10 players"
        echo "  • 2 games per player"
        echo "  • 10 second intervals"
        echo "  • Good for playing along"
        echo ""
        ENVIRONMENT="$ENVIRONMENT" \
        TOURNAMENT_ID="$TOURNAMENT_ID" \
        NUM_PLAYERS=10 \
        GAMES_PER_CYCLE=2 \
        BATCH_INTERVAL=10 \
        BATCHES_PER_GAME=5 \
        MOVES_PER_BATCH=10 \
        k6 run website/scripts/leaderboard-stress-test.ts
        ;;
    
    medium)
        echo -e "${BLUE}Medium Load Configuration:${NC}"
        echo "  • 30 players"
        echo "  • 3 games per player"
        echo "  • 5 second intervals"
        echo "  • Tests triggerer rotation"
        echo ""
        ENVIRONMENT="$ENVIRONMENT" \
        TOURNAMENT_ID="$TOURNAMENT_ID" \
        NUM_PLAYERS=30 \
        GAMES_PER_CYCLE=3 \
        BATCH_INTERVAL=5 \
        BATCHES_PER_GAME=5 \
        MOVES_PER_BATCH=10 \
        k6 run website/scripts/leaderboard-stress-test.ts
        ;;
    
    heavy)
        echo -e "${BLUE}Heavy Load Configuration:${NC}"
        echo "  • 50 players"
        echo "  • 5 games per player"
        echo "  • 3 second intervals"
        echo "  • High stress test"
        echo ""
        ENVIRONMENT="$ENVIRONMENT" \
        TOURNAMENT_ID="$TOURNAMENT_ID" \
        NUM_PLAYERS=50 \
        GAMES_PER_CYCLE=5 \
        BATCH_INTERVAL=3 \
        BATCHES_PER_GAME=8 \
        MOVES_PER_BATCH=15 \
        k6 run website/scripts/leaderboard-stress-test.ts
        ;;
    
    stress)
        echo -e "${BLUE}Maximum Stress Configuration:${NC}"
        echo "  • 100 players"
        echo "  • 5 games per player"
        echo "  • 3 second intervals"
        echo "  • Tests breaking point"
        echo ""
        ENVIRONMENT="$ENVIRONMENT" \
        TOURNAMENT_ID="$TOURNAMENT_ID" \
        NUM_PLAYERS=100 \
        GAMES_PER_CYCLE=5 \
        BATCH_INTERVAL=3 \
        BATCHES_PER_GAME=10 \
        MOVES_PER_BATCH=20 \
        k6 run website/scripts/leaderboard-stress-test.ts
        ;;
    
    custom)
        echo -e "${BLUE}Custom Configuration:${NC}"
        echo "Using environment variables..."
        echo "  NUM_PLAYERS=${NUM_PLAYERS:-20}"
        echo "  GAMES_PER_CYCLE=${GAMES_PER_CYCLE:-3}"
        echo "  BATCH_INTERVAL=${BATCH_INTERVAL:-5}"
        echo ""
        ENVIRONMENT="$ENVIRONMENT" \
        TOURNAMENT_ID="$TOURNAMENT_ID" \
        k6 run website/scripts/leaderboard-stress-test.ts
        ;;
    
    *)
        echo -e "${RED}❌ Unknown scenario: $SCENARIO${NC}"
        echo "Available scenarios: light, medium, heavy, stress, custom"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}✅ Test completed!${NC}"
