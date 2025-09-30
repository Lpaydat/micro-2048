#!/usr/bin/env python3
"""
Stress Test Coordinator - Sets up tournaments for high-load testing
Coordinates tournament creation and provides shared tournament info for K6 stress testing
"""

import requests
import json
import sys
import time
import random
import string
import argparse
from typing import Optional, Dict, List, Any

# ANSI color codes for output
RED = '\033[91m'
GREEN = '\033[92m'
YELLOW = '\033[93m'
BLUE = '\033[94m'
RESET = '\033[0m'
BOLD = '\033[1m'

class StressTestCoordinator:
    def __init__(self, chain_id: str, app_id: str, base_url: str = "http://localhost:8088"):
        self.base_url = base_url
        self.main_chain_id = chain_id
        self.app_id = app_id
        self.main_url = f"{base_url}/chains/{chain_id}/applications/{app_id}"
        
        # Coordinator credentials
        suffix = ''.join(random.choices(string.ascii_lowercase + string.digits, k=8))
        self.coordinator_username = f"coord_{suffix}"
        self.coordinator_password = f"pass_{suffix}"
        
        self.created_tournaments = []
        self.test_config = {}
    
    def log(self, message: str, color: str = RESET):
        """Print a colored log message"""
        print(f"{color}{message}{RESET}")
    
    def log_section(self, title: str):
        """Print a section header"""
        print(f"\n{BOLD}{YELLOW}{'='*70}")
        print(f"  {title}")
        print(f"{'='*70}{RESET}\n")
    
    def make_graphql_request(self, url: str, query: str, variables: Optional[Dict[str, Any]] = None, show_error: bool = True) -> Optional[Dict[str, Any]]:
        """Execute a GraphQL request"""
        headers = {"Content-Type": "application/json"}
        payload = {"query": query}
        if variables:
            payload["variables"] = variables
        
        try:
            response = requests.post(url, json=payload, headers=headers, timeout=30)
            
            try:
                data = response.json()
            except:
                data = None
            
            if data and "errors" in data and show_error:
                self.log(f"GraphQL Error: {data['errors'][0].get('message', 'Unknown error')}", RED)
                return None
            
            response.raise_for_status()
            return data
            
        except requests.exceptions.Timeout:
            if show_error:
                self.log("Request timed out", RED)
            return None
        except requests.exceptions.HTTPError as e:
            if show_error:
                self.log(f"HTTP Error {e.response.status_code}", RED)
                try:
                    error_data = e.response.json()
                    if "error" in error_data:
                        self.log(f"  Details: {error_data['error']}", RED)
                except:
                    pass
            return None
        except requests.exceptions.RequestException as e:
            if show_error:
                self.log(f"Request failed: {e}", RED)
            return None
    
    def register_coordinator(self) -> bool:
        """Register the coordinator player"""
        self.log_section("COORDINATOR SETUP")
        
        query = f'''
        mutation Register {{
            registerPlayer(username: "{self.coordinator_username}", passwordHash: "{self.coordinator_password}")
        }}
        '''
        
        self.log(f"Registering coordinator player '{self.coordinator_username}'...", BLUE)
        result = self.make_graphql_request(self.main_url, query)
        
        if result and "data" in result:
            self.log(f"✓ Coordinator registered successfully", GREEN)
            return True
        else:
            self.log(f"✗ Failed to register coordinator", RED)
            return False
    
    def create_stress_test_tournaments(self, num_tournaments: int = 3, shard_count: int = 8) -> bool:
        """Create multiple tournaments optimized for stress testing"""
        self.log_section(f"TOURNAMENT CREATION - {num_tournaments} Tournaments")
        
        import time as time_module
        current_time = int(time_module.time() * 1_000_000)  # Convert to microseconds
        
        # Create tournaments with simple numeric names
        tournament_configs = [
            {
                "name": f"{i}",
                "start_time": 0,  # No time constraints for active tournaments
                "end_time": 0,
                "shard_number": shard_count,
                "description": f"Tournament {i}"
            }
            for i in range(num_tournaments)
        ]
        
        for i, config in enumerate(tournament_configs):
            self.log(f"Creating tournament {i+1}/{len(tournament_configs)}: {config['name']}...", BLUE)
            self.log(f"  Shards: {config['shard_number']}, Description: {config['description']}", BLUE)
            
            query = f'''
            mutation CreateTournament {{
                leaderboardAction(
                    leaderboardId: ""
                    action: Create
                    settings: {{
                        name: "{config['name']}"
                        startTime: "{config['start_time']}"
                        endTime: "{config['end_time']}"
                        shardNumber: {config['shard_number']}
                    }}
                    player: "{self.coordinator_username}"
                    passwordHash: "{self.coordinator_password}"
                )
            }}
            '''
            
            result = self.make_graphql_request(self.main_url, query)
            
            if result and "data" in result:
                tournament_result = result['data'].get('leaderboardAction') if isinstance(result['data'], dict) else result['data']
                self.log(f"✓ Tournament '{config['name']}' created", GREEN)
                
                # Store tournament info
                self.created_tournaments.append({
                    'name': config['name'],
                    'host': self.coordinator_username,
                    'shard_count': config['shard_number'],
                    'description': config['description'],
                    'creation_result': tournament_result
                })
            else:
                self.log(f"✗ Failed to create tournament '{config['name']}'", RED)
                return False
            
            if i < len(tournament_configs) - 1:
                time.sleep(1)  # Brief wait between creations
        
        self.log(f"✓ Successfully created {len(self.created_tournaments)} tournaments", GREEN)
        return True
    
    def query_tournament_details(self) -> bool:
        """Query and store tournament details for K6 configuration"""
        self.log_section("TOURNAMENT DETAILS COLLECTION")
        
        query = f'''
        {{
            leaderboards {{
                leaderboardId
                name
                host
                chainId
                totalBoards
                shardIds
            }}
        }}
        '''
        
        self.log("Collecting tournament details for K6 configuration...", BLUE)
        result = self.make_graphql_request(self.main_url, query)
        
        if result and "data" in result and "leaderboards" in result["data"]:
            leaderboards = [lb for lb in result["data"]["leaderboards"] 
                           if lb["leaderboardId"] and lb["host"] == self.coordinator_username]
            
            if leaderboards:
                self.log(f"✓ Found {len(leaderboards)} coordinator tournaments:", GREEN)
                
                tournament_details = []
                for lb in leaderboards:
                    details = {
                        'id': lb['leaderboardId'],
                        'name': lb['name'],
                        'chainId': lb['chainId'],
                        'shardIds': lb.get('shardIds', []),
                        'shardCount': len(lb.get('shardIds', [])),
                        'totalBoards': lb.get('totalBoards', 0)
                    }
                    tournament_details.append(details)
                    
                    self.log(f"  - {details['name']}: {details['id'][:16]}...", GREEN)
                    self.log(f"    Chain: {details['chainId'][:16]}..., Shards: {details['shardCount']}", GREEN)
                
                # Store for K6 configuration
                self.test_config['tournaments'] = tournament_details
                self.test_config['coordinator'] = {
                    'username': self.coordinator_username,
                    'password': self.coordinator_password
                }
                self.test_config['api'] = {
                    'base_url': self.base_url,
                    'chain_id': self.main_chain_id,
                    'app_id': self.app_id,
                    'main_url': self.main_url
                }
                
                return True
            else:
                self.log("✗ No tournaments found for coordinator", RED)
                return False
        else:
            self.log("✗ Failed to query tournament details", RED)
            return False
    
    def save_test_config(self, output_file: str = "stress_test_config.json") -> bool:
        """Save test configuration for K6 stress test"""
        self.log_section("TEST CONFIGURATION EXPORT")
        
        # Add additional configuration for stress testing
        self.test_config['stress_test'] = {
            'total_players': 60,
            'ramp_up_duration': '3m',
            'sustained_duration': '8m',
            'ramp_down_duration': '2m',
            'boards_per_player': 2,
            'moves_per_board': 30,
            'move_batch_size': 15,
            'concurrent_tournaments': len(self.test_config.get('tournaments', [])),
            'recommended_players_per_tournament': 60 // max(len(self.test_config.get('tournaments', [])), 1)
        }
        
        # Add performance monitoring configuration
        self.test_config['monitoring'] = {
            'metrics_interval': '30s',
            'error_threshold': 5.0,  # 5% error rate threshold
            'response_time_threshold': '2s',
            'concurrent_request_limit': 50
        }
        
        try:
            with open(output_file, 'w') as f:
                json.dump(self.test_config, f, indent=2)
            
            self.log(f"✓ Test configuration saved to '{output_file}'", GREEN)
            self.log(f"  Tournaments: {len(self.test_config.get('tournaments', []))}", GREEN)
            self.log(f"  Target Players: {self.test_config['stress_test']['total_players']}", GREEN)
            self.log(f"  Recommended Distribution: ~{self.test_config['stress_test']['recommended_players_per_tournament']} players per tournament", GREEN)
            
            return True
        except Exception as e:
            self.log(f"✗ Failed to save configuration: {str(e)}", RED)
            return False
    
    def verify_tournament_accessibility(self) -> bool:
        """Verify tournaments are accessible for stress testing"""
        self.log_section("TOURNAMENT ACCESSIBILITY VERIFICATION")
        
        if not self.test_config.get('tournaments'):
            self.log("✗ No tournaments to verify", RED)
            return False
        
        all_accessible = True
        for tournament in self.test_config['tournaments']:
            tournament_id = tournament['id']
            chain_id = tournament['chainId']
            name = tournament['name']
            
            # Try to access tournament chain
            tournament_url = f"{self.base_url}/chains/{chain_id}/applications/{self.app_id}"
            
            query = '''
            {
                leaderboards {
                    leaderboardId
                    name
                    totalBoards
                    totalPlayers
                }
            }
            '''
            
            self.log(f"Verifying tournament '{name}' accessibility...", BLUE)
            result = self.make_graphql_request(tournament_url, query, show_error=False)
            
            if result and "data" in result:
                self.log(f"  ✓ Tournament '{name}' is accessible", GREEN)
            else:
                self.log(f"  ✗ Tournament '{name}' is not accessible", RED)
                all_accessible = False
        
        if all_accessible:
            self.log("✓ All tournaments are accessible for stress testing", GREEN)
        else:
            self.log("⚠ Some tournaments may not be accessible - check configuration", YELLOW)
        
        return all_accessible
    
    def run_coordinator_setup(self, num_tournaments: int = 3, shard_count: int = 8, output_file: str = "stress_test_config.json") -> bool:
        """Run complete coordinator setup"""
        self.log_section("STRESS TEST COORDINATOR")
        self.log(f"Setting up stress test infrastructure for {num_tournaments} tournaments", BLUE)
        self.log(f"Target: 30 concurrent players across tournaments", BLUE)
        self.log(f"Shard configuration: {shard_count} shards per tournament", BLUE)
        
        # Step 1: Register coordinator
        if not self.register_coordinator():
            self.log("\n❌ FAILED: Could not register coordinator", RED)
            return False
        time.sleep(1)
        
        # Step 2: Create tournaments
        if not self.create_stress_test_tournaments(num_tournaments, shard_count):
            self.log("\n❌ FAILED: Could not create tournaments", RED)
            return False
        time.sleep(2)  # Wait for tournaments to be processed
        
        # Step 3: Query tournament details
        if not self.query_tournament_details():
            self.log("\n❌ FAILED: Could not collect tournament details", RED)
            return False
        
        # Step 4: Save configuration for K6
        if not self.save_test_config(output_file):
            self.log("\n❌ FAILED: Could not save test configuration", RED)
            return False
        
        # Step 5: Verify accessibility
        if not self.verify_tournament_accessibility():
            self.log("\n⚠ WARNING: Some tournaments may not be accessible", YELLOW)
        
        # Success summary
        self.log_section("COORDINATOR SETUP COMPLETE")
        self.log("✅ Stress test infrastructure is ready!", GREEN)
        self.log(f"✓ Coordinator: {self.coordinator_username}", GREEN)
        self.log(f"✓ Tournaments: {len(self.created_tournaments)}", GREEN)
        self.log(f"✓ Configuration: {output_file}", GREEN)
        self.log("\n🚀 Ready to run K6 stress test with 30 concurrent players!", BOLD + GREEN)
        self.log(f"   Use: k6 run stress_test_k6.js", BLUE)
        
        return True


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Stress Test Coordinator - Setup tournaments for high-load testing")
    parser.add_argument("chain_id", help="Main chain ID")
    parser.add_argument("app_id", help="Application ID")
    parser.add_argument("--tournaments", "-t", type=int, default=3, help="Number of tournaments to create (default: 3)")
    parser.add_argument("--shards", "-s", type=int, default=8, help="Shards per tournament (default: 8)")
    parser.add_argument("--url", "-u", default="http://localhost:8088", help="Base URL (default: http://localhost:8088)")
    parser.add_argument("--output", "-o", default="stress_test_config.json", help="Output configuration file (default: stress_test_config.json)")
    
    args = parser.parse_args()
    
    # Validate arguments
    if args.tournaments < 1 or args.tournaments > 10:
        print(f"{RED}Error: Number of tournaments must be between 1 and 10{RESET}")
        sys.exit(1)
    
    if args.shards < 1 or args.shards > 32:
        print(f"{RED}Error: Shards per tournament must be between 1 and 32{RESET}")
        sys.exit(1)
    
    # Run coordinator setup
    coordinator = StressTestCoordinator(args.chain_id, args.app_id, args.url)
    success = coordinator.run_coordinator_setup(args.tournaments, args.shards, args.output)
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()