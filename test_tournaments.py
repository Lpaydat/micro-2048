#!/usr/bin/env python3
"""
Test script for tournament event reading functionality
Tests player registration, tournament creation, and board creation on multiple tournaments
"""

import requests
import json
import sys
import time
from typing import Optional, Dict, List, Any

# ANSI color codes for output
RED = '\033[91m'
GREEN = '\033[92m'
YELLOW = '\033[93m'
BLUE = '\033[94m'
RESET = '\033[0m'
BOLD = '\033[1m'

class TournamentTester:
    def __init__(self, chain_id: str, app_id: str, base_url: str = "http://localhost:8088"):
        self.base_url = base_url
        self.main_chain_id = chain_id
        self.app_id = app_id
        self.main_url = f"{base_url}/chains/{chain_id}/applications/{app_id}"
        self.player_chain_id = None
        self.tournaments = []
    
    def log(self, message: str, color: str = RESET):
        """Print a colored log message"""
        print(f"{color}{message}{RESET}")
    
    def log_section(self, title: str):
        """Print a section header"""
        print(f"\n{BOLD}{YELLOW}{'='*60}")
        print(f"  {title}")
        print(f"{'='*60}{RESET}\n")
    
    def make_graphql_request(self, url: str, query: str) -> Optional[Dict[str, Any]]:
        """Execute a GraphQL request"""
        headers = {"Content-Type": "application/json"}
        payload = {"query": query}
        
        try:
            response = requests.post(url, json=payload, headers=headers, timeout=10)
            response.raise_for_status()
            data = response.json()
            
            if "errors" in data:
                self.log(f"GraphQL Errors: {json.dumps(data['errors'], indent=2)}", RED)
                return None
            
            return data
            
        except requests.exceptions.Timeout:
            self.log("Request timed out", RED)
            return None
        except requests.exceptions.RequestException as e:
            self.log(f"Request failed: {e}", RED)
            return None
        except json.JSONDecodeError as e:
            self.log(f"Invalid JSON: {e}", RED)
            return None
    
    def register_player(self, username: str = "hello", password: str = "hello") -> bool:
        """Register a new player"""
        self.log_section("STEP 1: Register Player")
        
        query = f'''
        mutation Register {{
            registerPlayer(username: "{username}", passwordHash: "{password}")
        }}
        '''
        
        self.log(f"Registering player '{username}'...", BLUE)
        result = self.make_graphql_request(self.main_url, query)
        
        if result and "data" in result:
            self.log(f"✓ Player registered successfully", GREEN)
            self.log(f"  Chain ID: {result['data']['registerPlayer']}", GREEN)
            return True
        else:
            self.log(f"✗ Failed to register player", RED)
            return False
    
    def create_tournament(self, name: str = "test") -> Optional[str]:
        """Create a single tournament"""
        query = f'''
        mutation CreateTournament {{
            leaderboardAction(
                leaderboardId: ""
                action: Create
                settings: {{
                    name: "{name}"
                    startTime: "0"
                    endTime: "18588537542440000"
                    shardNumber: 2
                }}
                player: "hello"
                passwordHash: "hello"
            )
        }}
        '''
        
        result = self.make_graphql_request(self.main_url, query)
        
        if result and "data" in result:
            tournament_id = result['data']['leaderboardAction']
            return tournament_id
        return None
    
    def create_multiple_tournaments(self, count: int = 3) -> bool:
        """Create multiple tournaments"""
        self.log_section(f"STEP 2: Create {count} Tournaments")
        
        for i in range(1, count + 1):
            self.log(f"Creating tournament {i}/{count}...", BLUE)
            tournament_id = self.create_tournament(f"test_{i}")
            
            if tournament_id:
                self.log(f"✓ Tournament {i} created: {tournament_id}", GREEN)
                self.tournaments.append(tournament_id)
            else:
                self.log(f"✗ Failed to create tournament {i}", RED)
                return False
            
            if i < count:
                time.sleep(2)  # Wait between creations
        
        return True
    
    def query_state(self) -> Dict[str, Any]:
        """Query current game state"""
        self.log_section("STEP 3: Query Game State")
        
        query = '''
        {
            leaderboards {
                leaderboardId
                name
                host
                chainId
                totalBoards
                shardIds
            }
            player(username: "hello") {
                username
                chainId
                isMod
            }
            boards {
                boardId
                chainId
                shardId
                leaderboardId
            }
        }
        '''
        
        self.log("Querying leaderboards and player info...", BLUE)
        result = self.make_graphql_request(self.main_url, query)
        
        if result and "data" in result:
            data = result["data"]
            
            # Extract player chain
            if data.get("player"):
                self.player_chain_id = data["player"]["chainId"]
                self.log(f"✓ Found player chain: {self.player_chain_id}", GREEN)
            
            # Extract tournaments
            if data.get("leaderboards"):
                leaderboards = [lb for lb in data["leaderboards"] if lb["leaderboardId"]]
                self.log(f"✓ Found {len(leaderboards)} tournament(s):", GREEN)
                for lb in leaderboards:
                    self.log(f"  - {lb['leaderboardId']} ('{lb['name']}')", GREEN)
                return data
        else:
            self.log("✗ Failed to query state", RED)
            return {}
    
    def create_board(self, tournament_id: str, timestamp: str = "1000") -> bool:
        """Create a board for a specific tournament"""
        if not self.player_chain_id:
            self.log("Error: No player chain ID available", RED)
            return False
        
        player_url = f"{self.base_url}/chains/{self.player_chain_id}/applications/{self.app_id}"
        
        query = f'''
        mutation NewBoard {{
            newBoard(
                player: "hello"
                passwordHash: "hello"
                timestamp: "{timestamp}"
                leaderboardId: "{tournament_id}"
            )
        }}
        '''
        
        self.log(f"Creating board for tournament: {tournament_id[:16]}...", BLUE)
        result = self.make_graphql_request(player_url, query)
        
        if result and "data" in result:
            self.log(f"✓ Board created successfully", GREEN)
            return True
        else:
            self.log(f"✗ Failed to create board", RED)
            return False
    
    def test_board_creation(self) -> Dict[str, bool]:
        """Test board creation on all tournaments"""
        self.log_section("STEP 4: Test Board Creation")
        
        # Get fresh state
        state = self.query_state()
        if not state:
            return {}
        
        leaderboards = [lb for lb in state.get("leaderboards", []) if lb["leaderboardId"]]
        if not leaderboards:
            self.log("No tournaments found to test", RED)
            return {}
        
        results = {}
        timestamps = ["1000", "2000", "3000", "4000", "5000"]
        
        # Test each tournament
        for idx, lb in enumerate(leaderboards):
            tournament_id = lb["leaderboardId"]
            name = lb["name"]
            position = "First" if idx == 0 else "Last" if idx == len(leaderboards)-1 else f"#{idx+1}"
            
            self.log(f"\nTesting {position} tournament ({name}):", YELLOW)
            success = self.create_board(tournament_id, timestamps[idx % len(timestamps)])
            results[f"{position} ({name})"] = success
            
            if idx < len(leaderboards) - 1:
                time.sleep(2)
        
        return results
    
    def run_full_test(self) -> bool:
        """Run the complete test sequence"""
        self.log_section("Tournament Event Testing")
        self.log(f"Main Chain: {self.main_chain_id}", BLUE)
        self.log(f"App ID: {self.app_id}", BLUE)
        
        # Step 1: Register player
        if not self.register_player():
            return False
        time.sleep(2)
        
        # Step 2: Create tournaments
        if not self.create_multiple_tournaments(3):
            return False
        time.sleep(2)
        
        # Step 3: Initial state query
        self.query_state()
        
        # Step 4: Test board creation
        results = self.test_board_creation()
        
        # Step 5: Final state check
        self.log_section("STEP 5: Final State Check")
        final_state = self.query_state()
        
        if final_state and "boards" in final_state:
            boards = final_state.get("boards", [])
            self.log(f"Total boards created: {len(boards)}", BLUE)
        
        # Summary
        self.log_section("TEST SUMMARY")
        
        if results:
            all_passed = all(results.values())
            failed_count = sum(1 for v in results.values() if not v)
            
            for test_name, passed in results.items():
                status = f"{GREEN}✓ PASS{RESET}" if passed else f"{RED}✗ FAIL{RESET}"
                self.log(f"{test_name}: {status}")
            
            self.log("")
            if all_passed:
                self.log(f"Result: ALL TESTS PASSED ({len(results)}/{len(results)}) ✓", GREEN)
                return True
            else:
                self.log(f"Result: {failed_count} TEST(S) FAILED ✗", RED)
                return False
        else:
            self.log("No tests were run", YELLOW)
            return False


def main():
    """Main entry point"""
    if len(sys.argv) < 3:
        print(f"{YELLOW}Usage: python3 test_tournaments.py <CHAIN_ID> <APP_ID>{RESET}")
        print(f"{YELLOW}Example: python3 test_tournaments.py 363c9c77... 2519e58e...{RESET}")
        sys.exit(1)
    
    chain_id = sys.argv[1]
    app_id = sys.argv[2]
    
    tester = TournamentTester(chain_id, app_id)
    success = tester.run_full_test()
    
    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()