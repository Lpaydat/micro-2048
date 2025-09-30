#!/bin/bash

# U2048 Leaderboard Statistics Display
# Shows leaderboard update statistics after stress testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
RESET='\033[0m'

# Function to print colored output
log() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${RESET}"
}

# Configuration
DEFAULT_URL="http://localhost:8088"

# Function to display usage
usage() {
    echo "Usage: $0 <CHAIN_ID> <APP_ID> [OPTIONS]"
    echo
    echo "Required:"
    echo "  CHAIN_ID          Main chain ID for the U2048 application"
    echo "  APP_ID            Application ID for U2048"
    echo
    echo "Options:"
    echo "  -u, --url URL            Base URL for the service (default: $DEFAULT_URL)"
    echo "  -h, --help               Show this help message"
    echo
    echo "Examples:"
    echo "  $0 363c9c77... 2519e58e..."
    echo "  $0 363c9c77... 2519e58e... --url http://remote:8088"
    echo
}

# Main function
main() {
    local chain_id=""
    local app_id=""
    local url=$DEFAULT_URL
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -u|--url)
                url="$2"
                shift 2
                ;;
            -h|--help)
                usage
                exit 0
                ;;
            -*)
                log "${RED}" "Unknown option: $1"
                usage
                exit 1
                ;;
            *)
                if [ -z "$chain_id" ]; then
                    chain_id="$1"
                elif [ -z "$app_id" ]; then
                    app_id="$1"
                else
                    log "${RED}" "Too many arguments"
                    usage
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # Validate required parameters
    if [ -z "$chain_id" ] || [ -z "$app_id" ]; then
        log "${RED}" "❌ Chain ID and App ID are required"
        usage
        exit 1
    fi
    
    echo
    log "${BOLD}${YELLOW}" "=================================================================="
    log "${BOLD}${YELLOW}" "  U2048 LEADERBOARD STATISTICS"
    log "${BOLD}${YELLOW}" "=================================================================="
    echo
    
    local main_url="${url}/chains/${chain_id}/applications/${app_id}"
    
    log "${BLUE}" "📊 Fetching leaderboard statistics..."
    echo "   • Service URL: $main_url"
    echo "   • Chain ID: ${chain_id:0:16}..."
    echo "   • App ID: ${app_id:0:16}..."
    echo
    
    # GraphQL query for leaderboard stats
    local query='{"query": "query GetLeaderboardStats { leaderboard { leaderboardId name totalPlayers totalBoards } }"}'
    
    # Make the request
    local response=$(curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$query" \
        "$main_url" 2>/dev/null)
    
    if [ $? -ne 0 ] || [ -z "$response" ]; then
        log "${RED}" "❌ Failed to fetch leaderboard statistics"
        echo "   Make sure the service is running at: $url"
        exit 1
    fi
    
    # Check if jq is available for JSON parsing
    if command -v jq &> /dev/null; then
        # Parse response with jq
        local leaderboard=$(echo "$response" | jq -r '.data.leaderboard // empty')
        
        if [ -z "$leaderboard" ] || [ "$leaderboard" = "null" ]; then
            log "${RED}" "❌ No leaderboard data found"
            echo "   Response: $response"
            exit 1
        fi
        
        local tournament_name=$(echo "$leaderboard" | jq -r '.name // "Unknown"')
        local total_players=$(echo "$leaderboard" | jq -r '.totalPlayers // 0')
        local total_boards=$(echo "$leaderboard" | jq -r '.totalBoards // 0')
        
        log "${GREEN}" "✅ Leaderboard Statistics Retrieved"
        echo
        log "${BOLD}" "🏆 Tournament: $tournament_name"
        echo
        log "${BLUE}" "📈 Tournament Statistics:"
        echo "   • Total Players: $total_players"
        echo "   • Total Boards: $total_boards"
        
        if [ "$total_boards" != "0" ] && [ "$total_players" != "0" ]; then
            local boards_per_player=$(echo "scale=1; $total_boards / $total_players" | bc 2>/dev/null || echo "N/A")
            echo "   • Boards per Player: ${boards_per_player}"
        fi
        
        echo
        log "${BLUE}" "📝 Recent Update Log:"
        if [ -n "$update_log" ]; then
            echo "$update_log" | tail -10 | while IFS= read -r line; do
                echo "   • $line"
            done
        else
            echo "   • No updates recorded"
        fi
        
    else
        # Basic parsing without jq
        log "${YELLOW}" "⚠️  jq not available - showing raw response:"
        echo "$response" | python3 -m json.tool 2>/dev/null || echo "$response"
    fi
    
    echo
    log "${GREEN}" "📊 Statistics display completed"
}

# Check if script is being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi