#!/bin/bash

# U2048 Comprehensive Stress Test Runner
# Orchestrates the complete stress testing process with 200 concurrent players

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
RESET='\033[0m'

# Configuration
DEFAULT_TOURNAMENTS=3
DEFAULT_SHARDS=8
DEFAULT_URL="http://localhost:8088"
COORDINATOR_SCRIPT="stress_test_coordinator.py"
K6_SCRIPT="stress_test_k6.js"
CONFIG_FILE="stress_test_config.json"
RESULTS_DIR="stress_test_results"

# Function to print colored output
log() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${RESET}"
}

log_section() {
    local title=$1
    echo
    log "${BOLD}${YELLOW}" "=================================================================="
    log "${BOLD}${YELLOW}" "  $title"
    log "${BOLD}${YELLOW}" "=================================================================="
    echo
}

# Function to check if required tools are available
check_dependencies() {
    log_section "DEPENDENCY CHECK"
    
    local missing_deps=0
    
    # Check Python
    if ! command -v python3 &> /dev/null; then
        log "${RED}" "❌ Python3 is required but not installed"
        missing_deps=$((missing_deps + 1))
    else
        log "${GREEN}" "✓ Python3 found: $(python3 --version)"
    fi
    
    # Check K6
    if ! command -v k6 &> /dev/null; then
        log "${RED}" "❌ K6 is required but not installed"
        log "${YELLOW}" "   Install from: https://k6.io/docs/get-started/installation/"
        missing_deps=$((missing_deps + 1))
    else
        log "${GREEN}" "✓ K6 found: $(k6 version)"
    fi
    
    # Check Python requests module
    if ! python3 -c "import requests" &> /dev/null; then
        log "${RED}" "❌ Python 'requests' module is required"
        log "${YELLOW}" "   Install with: pip3 install requests"
        missing_deps=$((missing_deps + 1))
    else
        log "${GREEN}" "✓ Python requests module available"
    fi
    
    if [ $missing_deps -gt 0 ]; then
        log "${RED}" "❌ $missing_deps dependencies missing. Please install them first."
        exit 1
    fi
    
    log "${GREEN}" "✅ All dependencies satisfied"
}

# Function to validate input parameters
validate_parameters() {
    local chain_id=$1
    local app_id=$2
    
    if [ -z "$chain_id" ] || [ -z "$app_id" ]; then
        log "${RED}" "❌ Chain ID and App ID are required"
        return 1
    fi
    
    if [ ${#chain_id} -lt 16 ]; then
        log "${RED}" "❌ Chain ID appears to be too short"
        return 1
    fi
    
    if [ ${#app_id} -lt 16 ]; then
        log "${RED}" "❌ App ID appears to be too short"
        return 1
    fi
    
    return 0
}

# Function to setup test environment
setup_test_environment() {
    log_section "TEST ENVIRONMENT SETUP"
    
    # Create results directory
    if [ ! -d "$RESULTS_DIR" ]; then
        mkdir -p "$RESULTS_DIR"
        log "${GREEN}" "✓ Created results directory: $RESULTS_DIR"
    else
        log "${BLUE}" "→ Results directory already exists: $RESULTS_DIR"
    fi
    
    # Generate timestamp for this test run
    TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
    TEST_RUN_DIR="$RESULTS_DIR/run_$TIMESTAMP"
    mkdir -p "$TEST_RUN_DIR"
    
    log "${GREEN}" "✓ Test run directory: $TEST_RUN_DIR"
}

# Function to run coordinator setup
run_coordinator() {
    local chain_id=$1
    local app_id=$2
    local tournaments=$3
    local shards=$4
    local url=$5
    
    log_section "TOURNAMENT COORDINATOR"
    log "${BLUE}" "→ Setting up $tournaments tournaments with $shards shards each"
    log "${BLUE}" "→ Target: 60 peak concurrent players with fresh boards each game"
    
    # Run Python coordinator
    if python3 "$COORDINATOR_SCRIPT" \
        "$chain_id" \
        "$app_id" \
        --tournaments "$tournaments" \
        --shards "$shards" \
        --url "$url" \
        --output "$CONFIG_FILE"; then
        
        log "${GREEN}" "✅ Tournament coordinator completed successfully"
        
        # Copy config to results directory
        cp "$CONFIG_FILE" "$TEST_RUN_DIR/"
        
        return 0
    else
        log "${RED}" "❌ Tournament coordinator failed"
        return 1
    fi
}

# Function to run K6 stress test
run_k6_stress_test() {
    log_section "K6 STRESS TEST EXECUTION"
    log "${BLUE}" "→ Starting stress test with fresh boards per game"
    log "${YELLOW}" "⚠️  This is a high-load test - monitor system resources"
    
    # Check if config file exists
    if [ ! -f "$CONFIG_FILE" ]; then
        log "${RED}" "❌ Configuration file '$CONFIG_FILE' not found"
        log "${RED}" "   Make sure coordinator setup completed successfully"
        return 1
    fi
    
    # Display test configuration summary
    if command -v jq &> /dev/null; then
        log "${BLUE}" "📊 Test Configuration Summary:"
        echo "   Players: $(jq -r '.stress_test.total_players' "$CONFIG_FILE")"
        echo "   Tournaments: $(jq -r '.tournaments | length' "$CONFIG_FILE")"
        echo "   Boards per Player: $(jq -r '.stress_test.boards_per_player' "$CONFIG_FILE")"
        echo "   Move Batch Size: $(jq -r '.stress_test.move_batch_size' "$CONFIG_FILE")"
    fi
    
    echo
    log "${YELLOW}" "Starting K6 stress test in 5 seconds..."
    sleep 5
    
    # Run K6 with detailed output
    local k6_output="$TEST_RUN_DIR/k6_output.log"
    local k6_results="$TEST_RUN_DIR/stress_test_results.json"
    
    if k6 run "$K6_SCRIPT" \
        --out json="$k6_results" \
        2>&1 | tee "$k6_output"; then
        
        log "${GREEN}" "✅ K6 stress test completed"
        
        # Move results file if K6 created it in current directory
        if [ -f "stress_test_results.json" ] && [ ! -f "$k6_results" ]; then
            mv "stress_test_results.json" "$k6_results"
        fi
        
        return 0
    else
        log "${RED}" "❌ K6 stress test failed or was interrupted"
        return 1
    fi
}

# Function to analyze results
analyze_results() {
    log_section "RESULT ANALYSIS"
    
    local k6_results="$TEST_RUN_DIR/stress_test_results.json"
    
    if [ -f "$k6_results" ] && command -v jq &> /dev/null; then
        log "${BLUE}" "📊 Performance Summary:"
        
        # Extract key metrics using jq
        local total_requests=$(jq -r '.metrics.http_reqs.values.count // 0' "$k6_results")
        local request_rate=$(jq -r '.metrics.http_reqs.values.rate // 0' "$k6_results")
        local error_rate=$(jq -r '.metrics.http_req_failed.values.rate // 0' "$k6_results")
        local avg_duration=$(jq -r '.metrics.http_req_duration.values.avg // 0' "$k6_results")
        local p95_duration=$(jq -r '.metrics.http_req_duration.values["p(95)"] // 0' "$k6_results")
        
        echo "   • Total Requests: $total_requests"
        echo "   • Request Rate: $(printf '%.2f' "$request_rate") req/s"
        echo "   • Error Rate: $(printf '%.2f' "$(echo "$error_rate * 100" | bc)")%"
        echo "   • Avg Response Time: $(printf '%.2f' "$avg_duration")ms"
        echo "   • 95th Percentile: $(printf '%.2f' "$p95_duration")ms"
        
        # Success criteria check
        local error_percentage=$(echo "$error_rate * 100" | bc)
        local error_pass=$(echo "$error_percentage < 5" | bc)
        local p95_pass=$(echo "$p95_duration < 2000" | bc)
        
        echo
        log "${BLUE}" "🎯 Success Criteria:"
        if [ "$error_pass" = "1" ]; then
            log "${GREEN}" "   ✅ Error rate < 5%: PASS"
        else
            log "${RED}" "   ❌ Error rate < 5%: FAIL"
        fi
        
        if [ "$p95_pass" = "1" ]; then
            log "${GREEN}" "   ✅ 95th percentile < 2s: PASS"
        else
            log "${RED}" "   ❌ 95th percentile < 2s: FAIL"
        fi
        
        if [ "$error_pass" = "1" ] && [ "$p95_pass" = "1" ]; then
            log "${GREEN}" "🏆 Overall Result: PASS"
        else
            log "${RED}" "💥 Overall Result: FAIL"
        fi
        
    else
        log "${YELLOW}" "⚠️  Detailed analysis unavailable (missing jq or results file)"
        log "${BLUE}" "→ Check log files in: $TEST_RUN_DIR"
    fi
    
    echo
    log "${BLUE}" "📁 All test artifacts saved to: $TEST_RUN_DIR"
    echo "   • stress_test_config.json - Tournament configuration"
    echo "   • k6_output.log - K6 execution log"
    echo "   • stress_test_results.json - Detailed metrics"
}

# Function to cleanup
cleanup() {
    log_section "CLEANUP"
    
    # Remove temporary config file from current directory
    if [ -f "$CONFIG_FILE" ]; then
        rm "$CONFIG_FILE"
        log "${GREEN}" "✓ Cleaned up temporary configuration file"
    fi
    
    log "${GREEN}" "✓ Cleanup completed"
}

# Function to display usage
usage() {
    echo "Usage: $0 <CHAIN_ID> <APP_ID> [OPTIONS]"
    echo
    echo "Required:"
    echo "  CHAIN_ID          Main chain ID for the U2048 application"
    echo "  APP_ID            Application ID for U2048"
    echo
    echo "Options:"
    echo "  -t, --tournaments NUM    Number of tournaments to create (default: $DEFAULT_TOURNAMENTS)"
    echo "  -s, --shards NUM         Shards per tournament (default: $DEFAULT_SHARDS)"
    echo "  -u, --url URL            Base URL for the service (default: $DEFAULT_URL)"
    echo "  --original               Use original single-phase test (less stable)"
    echo "  -h, --help               Show this help message"
    echo
    echo "Examples:"
    echo "  $0 363c9c77... 2519e58e..."
    echo "  $0 363c9c77... 2519e58e... --tournaments 5 --shards 16"
    echo "  $0 363c9c77... 2519e58e... --url http://remote:8088"
    echo
    echo "This script will:"
    echo "  1. Set up tournaments for stress testing"
    echo "  2. Run K6 stress test with fresh boards per game"
    echo "  3. Generate comprehensive performance reports"
    echo
}

# Main function
main() {
    local chain_id=""
    local app_id=""
    local tournaments=$DEFAULT_TOURNAMENTS
    local shards=$DEFAULT_SHARDS
    local url=$DEFAULT_URL
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--tournaments)
                tournaments="$2"
                shift 2
                ;;
            -s|--shards)
                shards="$2"
                shift 2
                ;;
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
    if ! validate_parameters "$chain_id" "$app_id"; then
        usage
        exit 1
    fi
    
    # Validate numeric parameters
    if ! [[ "$tournaments" =~ ^[0-9]+$ ]] || [ "$tournaments" -lt 1 ] || [ "$tournaments" -gt 10 ]; then
        log "${RED}" "❌ Tournaments must be a number between 1 and 10"
        exit 1
    fi
    
    if ! [[ "$shards" =~ ^[0-9]+$ ]] || [ "$shards" -lt 1 ] || [ "$shards" -gt 32 ]; then
        log "${RED}" "❌ Shards must be a number between 1 and 32"
        exit 1
    fi
    
    # Display test configuration
    log_section "U2048 STRESS TEST RUNNER"
    log "${BLUE}" "🎯 Test Configuration:"
    echo "   • Chain ID: ${chain_id:0:16}..."
    echo "   • App ID: ${app_id:0:16}..."
    echo "   • Tournaments: $tournaments"
    echo "   • Shards per Tournament: $shards"
    echo "   • Service URL: $url"
    echo "   • Target Players: 60 concurrent (peak)"
    echo "   • Test Mode: Fresh board per game"
    echo
    
    # Check dependencies
    check_dependencies
    
    # Setup test environment
    setup_test_environment
    
    # Set trap for cleanup on exit
    trap cleanup EXIT
    
    # Run coordinator setup
    if ! run_coordinator "$chain_id" "$app_id" "$tournaments" "$shards" "$url"; then
        log "${RED}" "❌ Stress test failed during coordinator setup"
        exit 1
    fi
    
    # Run K6 stress test
    if ! run_k6_stress_test; then
        log "${RED}" "❌ Stress test failed during K6 execution"
        exit 1
    fi
    
    # Analyze results
    analyze_results
    
    # Show leaderboard statistics
    log_section "LEADERBOARD STATISTICS"
    log "${BLUE}" "📊 Fetching final leaderboard update statistics..."
    echo
    
    if ./show_leaderboard_stats.sh "$chain_id" "$app_id" --url "$url"; then
        log "${GREEN}" "✅ Leaderboard statistics displayed successfully"
    else
        log "${YELLOW}" "⚠️  Could not fetch leaderboard statistics"
    fi
    
    # Final success message
    log_section "STRESS TEST COMPLETE"
    log "${GREEN}" "🎉 Stress test execution completed successfully!"
    log "${BLUE}" "📊 Check $TEST_RUN_DIR for detailed results"
    echo
}

# Check if script is being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi