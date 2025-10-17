#!/bin/bash

# ========================================
# 2048 Simulation Test Runner
# ========================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
ENVIRONMENT="production"
PERSONALITY="mixed"
GAMES_PER_BOT="3"
MOVES_PER_GAME="50"
VUS="20"
DURATION="30m"
TOURNAMENT_ID=""

# ========================================
# HELP FUNCTION
# ========================================

show_help() {
    echo -e "${BLUE}🤖 2048 Simulation Test Runner${NC}"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -e, --environment ENV     Environment: local, production (default: production)"
    echo "  -p, --personality TYPE    Bot personality: aggressive, strategic, casual, mixed (default: mixed)"
    echo "  -g, --games COUNT         Games per bot (default: 3)"
    echo "  -m, --moves COUNT         Moves per game (default: 50)"
    echo "  -v, --vus COUNT           Number of virtual users (default: 20)"
    echo "  -d, --duration DURATION   Test duration (default: 30m)"
    echo "  -t, --tournament ID        Specific tournament ID"
    echo "  -h, --help                Show this help message"
    echo ""
    echo "Presets:"
    echo "  --play-along              Play alongside bots (recommended)"
    echo "  --local-test              Local development testing"
    echo "  --stress-test             Stress testing"
    echo "  --aggressive              Aggressive bots only"
    echo "  --strategic               Strategic bots only"
    echo ""
    echo "Examples:"
    echo "  $0 --play-along"
    echo "  $0 --local-test"
    echo "  $0 --stress-test"
    echo "  $0 -e local -v 10 -d 5m"
    echo ""
}

# ========================================
# PRESET FUNCTIONS
# ========================================

preset_play_along() {
    ENVIRONMENT="production"
    PERSONALITY="mixed"
    GAMES_PER_BOT="3"
    MOVES_PER_GAME="50"
    VUS="20"
    DURATION="30m"
    echo -e "${GREEN}🎮 Play Along Preset: Realistic bots for human interaction${NC}"
}

preset_local_test() {
    ENVIRONMENT="local"
    PERSONALITY="mixed"
    GAMES_PER_BOT="2"
    MOVES_PER_GAME="20"
    VUS="8"
    DURATION="5m"
    echo -e "${GREEN}🔧 Local Test Preset: Light load for development${NC}"
}

preset_stress_test() {
    ENVIRONMENT="production"
    PERSONALITY="mixed"
    GAMES_PER_BOT="5"
    MOVES_PER_GAME="80"
    VUS="100"
    DURATION="1h"
    echo -e "${GREEN}🔥 Stress Test Preset: High load performance testing${NC}"
}

preset_aggressive() {
    ENVIRONMENT="production"
    PERSONALITY="aggressive"
    GAMES_PER_BOT="5"
    MOVES_PER_GAME="100"
    VUS="30"
    DURATION="15m"
    echo -e "${GREEN}🚀 Aggressive Preset: Fast-playing high activity bots${NC}"
}

preset_strategic() {
    ENVIRONMENT="production"
    PERSONALITY="strategic"
    GAMES_PER_BOT="2"
    MOVES_PER_GAME="40"
    VUS="25"
    DURATION="20m"
    echo -e "${GREEN}🧠 Strategic Preset: Thoughtful realistic gameplay${NC}"
}

# ========================================
# PARSE ARGUMENTS
# ========================================

while [[ $# -gt 0 ]]; do
    case $1 in
        -e|--environment)
            ENVIRONMENT="$2"
            shift 2
            ;;
        -p|--personality)
            PERSONALITY="$2"
            shift 2
            ;;
        -g|--games)
            GAMES_PER_BOT="$2"
            shift 2
            ;;
        -m|--moves)
            MOVES_PER_GAME="$2"
            shift 2
            ;;
        -v|--vus)
            VUS="$2"
            shift 2
            ;;
        -d|--duration)
            DURATION="$2"
            shift 2
            ;;
        -t|--tournament)
            TOURNAMENT_ID="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        --play-along)
            preset_play_along
            shift
            ;;
        --local-test)
            preset_local_test
            shift
            ;;
        --stress-test)
            preset_stress_test
            shift
            ;;
        --aggressive)
            preset_aggressive
            shift
            ;;
        --strategic)
            preset_strategic
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            show_help
            exit 1
            ;;
    esac
done

# ========================================
# VALIDATION
# ========================================

if [[ "$ENVIRONMENT" != "local" && "$ENVIRONMENT" != "production" ]]; then
    echo -e "${RED}Error: Environment must be 'local' or 'production'${NC}"
    exit 1
fi

if [[ "$PERSONALITY" != "aggressive" && "$PERSONALITY" != "strategic" && "$PERSONALITY" != "casual" && "$PERSONALITY" != "mixed" ]]; then
    echo -e "${RED}Error: Personality must be one of: aggressive, strategic, casual, mixed${NC}"
    exit 1
fi

# ========================================
# DISPLAY CONFIGURATION
# ========================================

echo -e "${BLUE}🚀 Starting 2048 Simulation Test${NC}"
echo ""
echo "Configuration:"
echo "  Environment: $ENVIRONMENT"
echo "  Bot Personality: $PERSONALITY"
echo "  Games per Bot: $GAMES_PER_BOT"
echo "  Moves per Game: $MOVES_PER_GAME"
echo "  Virtual Users: $VUS"
echo "  Duration: $DURATION"
if [[ -n "$TOURNAMENT_ID" ]]; then
    echo "  Tournament ID: $TOURNAMENT_ID"
fi
echo ""

# ========================================
# ENVIRONMENT CHECK
# ========================================

if [[ "$ENVIRONMENT" == "local" ]]; then
    echo -e "${YELLOW}🔧 Checking local environment...${NC}"
    
    # Check if localhost:8080 is accessible
    if ! curl -s --connect-timeout 5 "http://localhost:8080" > /dev/null; then
        echo -e "${RED}Error: Local server not accessible at http://localhost:8080${NC}"
        echo "Please ensure your local development server is running."
        exit 1
    fi
    
    echo -e "${GREEN}✅ Local environment is ready${NC}"
fi

# ========================================
# BUILD COMMAND
# ========================================

ENV_VARS="ENVIRONMENT=$ENVIRONMENT"
ENV_VARS="$ENV_VARS BOT_PERSONALITY=$PERSONALITY"
ENV_VARS="$ENV_VARS GAMES_PER_BOT=$GAMES_PER_BOT"
ENV_VARS="$ENV_VARS MOVES_PER_GAME=$MOVES_PER_GAME"

if [[ -n "$TOURNAMENT_ID" ]]; then
    ENV_VARS="$ENV_VARS TOURNAMENT_ID=$TOURNAMENT_ID"
fi

COMMAND="$ENV_VARS k6 run --vus $VUS --duration $DURATION website/scripts/simulation.ts"

# ========================================
# RUN SIMULATION
# ========================================

echo -e "${BLUE}🎮 Running simulation...${NC}"
echo -e "${YELLOW}Command: $COMMAND${NC}"
echo ""

# Check if k6 is installed
if ! command -v k6 &> /dev/null; then
    echo -e "${RED}Error: k6 is not installed${NC}"
    echo "Please install k6: https://k6.io/docs/getting-started/installation/"
    exit 1
fi

# Run the simulation
eval $COMMAND

echo ""
echo -e "${GREEN}✅ Simulation completed!${NC}"