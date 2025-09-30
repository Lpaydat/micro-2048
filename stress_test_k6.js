import http from "k6/http";
import { check, sleep } from "k6";
import { Rate, Counter, Trend } from "k6/metrics";
import { SharedArray } from "k6/data";
import exec from "k6/execution";

// ============================================================================
// CONFIGURATION
// ============================================================================

const testConfig = JSON.parse(open("./stress_test_config.json"));

const config = testConfig.stress_test || {
  total_players: 60,
  boards_per_player: 5,
  moves_per_board: 50, // Reduced from 800 to avoid timeouts
};

const apiConfig = testConfig.api;

const tournaments = new SharedArray("tournaments", function () {
  return testConfig.tournaments || [];
});

// Shared array to store registered players (will be populated in phase 1)
const registeredPlayers = new SharedArray("registered_players", function () {
  // This will be empty initially, populated during registration phase
  return [];
});

// ============================================================================
// METRICS
// ============================================================================

const errorRate = new Rate("errors");
const playerRegistrations = new Counter("player_registrations");
const boardCreations = new Counter("board_creations");
const moveOperations = new Counter("move_operations");
const boardResponseTime = new Trend("board_response_time");
const moveResponseTime = new Trend("move_response_time");

// ============================================================================
// K6 OPTIONS - TWO PHASE APPROACH
// ============================================================================

export const options = {
  scenarios: {
    // PHASE 1: Mass registration of all 60 players
    registration_phase: {
      executor: "shared-iterations",
      vus: 60,
      iterations: 60,
      maxDuration: "5m",
      startTime: "0s",
      exec: "registrationPhase",
    },
    
    // PHASE 2: All 60 players play games concurrently (starts after registration)
    gameplay_phase: {
      executor: "constant-vus",
      vus: 60,
      duration: "15m",
      startTime: "60s", // Start 60 seconds (1 minute) after test begins
      exec: "gameplayPhase",
    },
  },
  thresholds: {
    http_req_duration: ["p(95)<2000"],
    http_req_failed: ["rate<0.05"],
    errors: ["rate<0.05"],
  },
  batch: 20,
  batchPerHost: 10,
};

// ============================================================================
// GRAPHQL QUERIES
// ============================================================================

const QUERIES = {
  registerPlayer: `
    mutation RegisterPlayer($username: String!, $passwordHash: String!) {
      registerPlayer(username: $username, passwordHash: $passwordHash)
    }
  `,
  
  getPlayerData: `
    query GetPlayers {
      players {
        username
        chainId
      }
    }
  `,
  
  newBoard: `
    mutation NewBoard($player: String!, $passwordHash: String!, $timestamp: String!, $leaderboardId: String!) {
      newBoard(
        player: $player
        passwordHash: $passwordHash
        timestamp: $timestamp
        leaderboardId: $leaderboardId
      )
    }
  `,
  
  getBoards: `
    query GetBoards {
      boards {
        boardId
        player
        leaderboardId
      }
    }
  `,
  
  makeMoves: `
    mutation MakeMoves($boardId: String!, $player: String!, $passwordHash: String!, $moves: String!) {
      makeMoves(
        boardId: $boardId
        player: $player
        passwordHash: $passwordHash
        moves: $moves
      )
    }
  `,
};

// ============================================================================
// UTILITIES
// ============================================================================

function generateRandomString(length) {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let result = "";
  for (let i = 0; i < length; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}

function generateMoveBatch(count) {
  const directions = ["Up", "Right", "Down", "Left"];
  const baseTimestamp = Date.now();
  
  const moves = [];
  for (let i = 0; i < count; i++) {
    moves.push([
      directions[Math.floor(Math.random() * directions.length)],
      (baseTimestamp + i).toString(),
    ]);
  }
  
  return JSON.stringify(moves);
}

function makeGraphQLRequest(url, query, variables = null, metricName = null) {
  const payload = {
    query: query,
    ...(variables && { variables }),
  };
  
  const params = {
    headers: {
      "Content-Type": "application/json",
    },
    timeout: "30s",
  };
  
  const startTime = Date.now();
  const response = http.post(url, JSON.stringify(payload), params);
  const duration = Date.now() - startTime;
  
  if (metricName === "board") {
    boardResponseTime.add(duration);
  } else if (metricName === "move") {
    moveResponseTime.add(duration);
  }
  
  const success = check(response, {
    "status is 200": (r) => r.status === 200,
  });
  
  if (!success) {
    errorRate.add(1);
    return null;
  }
  
  errorRate.add(0);
  return response;
}

function selectTournament() {
  if (tournaments.length === 0) return null;
  const index = Math.floor(Math.random() * tournaments.length);
  return tournaments[index];
}

// ============================================================================
// PHASE 1: REGISTRATION
// ============================================================================

export function registrationPhase() {
  const vuNum = exec.vu.idInTest;
  const timestamp = Date.now();
  const random = generateRandomString(8);
  const username = `player_${vuNum}_${timestamp}_${random}`;
  const password = username; // Use username as password
  
  const mainUrl = `${apiConfig.base_url}/chains/${apiConfig.chain_id}/applications/${apiConfig.app_id}`;
  
  console.log(`🔐 [Phase 1 - VU ${vuNum}] Registering player: ${username}`);
  
  // Register player on main chain
  const response = makeGraphQLRequest(
    mainUrl,
    QUERIES.registerPlayer,
    { username, passwordHash: password }
  );
  
  if (response) {
    playerRegistrations.add(1);
    console.log(`✅ [Phase 1 - VU ${vuNum}] Player registered: ${username}`);
    
    // Store player data in a way that can be retrieved in phase 2
    // Note: K6 SharedArrays are immutable, so we'll use a different approach
    // We'll retrieve player data at the start of phase 2
  } else {
    console.error(`❌ [Phase 1 - VU ${vuNum}] Registration failed for ${username}`);
  }
  
  sleep(0.5);
}

// ============================================================================
// PHASE 2: GAMEPLAY
// ============================================================================

export function gameplayPhase() {
  const vuNum = exec.vu.idInTest;
  const mainUrl = `${apiConfig.base_url}/chains/${apiConfig.chain_id}/applications/${apiConfig.app_id}`;
  
  // Get all players from main chain
  const playersResponse = makeGraphQLRequest(mainUrl, QUERIES.getPlayerData);
  if (!playersResponse) {
    console.error(`❌ [Phase 2 - VU ${vuNum}] Failed to get players list`);
    return;
  }
  
  let allPlayers = [];
  try {
    const data = JSON.parse(playersResponse.body);
    allPlayers = data.data.players.filter(p => p.username.startsWith("player_"));
  } catch (e) {
    console.error(`❌ [Phase 2 - VU ${vuNum}] Failed to parse players: ${e}`);
    return;
  }
  
  if (allPlayers.length === 0) {
    console.error(`❌ [Phase 2 - VU ${vuNum}] No players found`);
    return;
  }
  
  // Each VU picks a player (distribute evenly)
  const playerIndex = (vuNum - 1) % allPlayers.length;
  const player = allPlayers[playerIndex];
  const password = player.username; // Password is same as username
  
  console.log(`🎮 [Phase 2 - VU ${vuNum}] Playing as ${player.username} on chain ${player.chainId.substring(0, 8)}...`);
  
  const playerUrl = `${apiConfig.base_url}/chains/${player.chainId}/applications/${apiConfig.app_id}`;
  
  // Select tournament
  const tournament = selectTournament();
  if (!tournament) {
    console.error(`❌ [Phase 2 - VU ${vuNum}] No tournament available`);
    return;
  }
  
  console.log(`🏆 [Phase 2 - VU ${vuNum}] Joined tournament: ${tournament.name}`);
  
  // STEP 1: Create multiple boards first (like test.ts lines 165-185)
  const boardsToCreate = config.boards_per_player;
  console.log(`📋 [Phase 2 - VU ${vuNum}] Creating ${boardsToCreate} boards...`);
  
  for (let i = 0; i < boardsToCreate; i++) {
    const timestamp = Date.now().toString();
    const boardResponse = makeGraphQLRequest(
      playerUrl,
      QUERIES.newBoard,
      {
        player: player.username,
        passwordHash: password,
        timestamp: timestamp,
        leaderboardId: tournament.id,
      },
      "board"
    );
    
    if (boardResponse) {
      boardCreations.add(1);
      console.log(`✅ [Phase 2 - VU ${vuNum}] Created board ${i + 1}/${boardsToCreate}`);
    } else {
      console.error(`❌ [Phase 2 - VU ${vuNum}] Failed to create board ${i + 1}`);
    }
    
    sleep(2); // Wait between board creations
  }
  
  sleep(2); // Wait for all boards to be ready
  
  // STEP 2: Get all board IDs for this player
  const boardsResponse = makeGraphQLRequest(playerUrl, QUERIES.getBoards);
  if (!boardsResponse) {
    console.error(`❌ [Phase 2 - VU ${vuNum}] Failed to get boards list`);
    return;
  }
  
  let boardIds = [];
  try {
    const boardsData = JSON.parse(boardsResponse.body);
    const playerBoards = boardsData.data.boards.filter(
      (b) => b.player === player.username && b.leaderboardId === tournament.id
    );
    boardIds = playerBoards.map(b => b.boardId);
  } catch (e) {
    console.error(`❌ [Phase 2 - VU ${vuNum}] Failed to parse boards: ${e}`);
    return;
  }
  
  if (boardIds.length === 0) {
    console.error(`❌ [Phase 2 - VU ${vuNum}] No board IDs found`);
    return;
  }
  
  console.log(`🎯 [Phase 2 - VU ${vuNum}] Got ${boardIds.length} boards, starting continuous gameplay...`);
  
  // STEP 3: Continuous gameplay loop - submit moves to boards (like test.ts lines 210-248)
  const MOVES_PER_BATCH = 25; // Small batches to avoid ending game too quickly
  const BATCHES_PER_BOARD = Math.ceil(config.moves_per_board / MOVES_PER_BATCH);
  
  for (const boardId of boardIds) {
    for (let batchIndex = 0; batchIndex < BATCHES_PER_BOARD; batchIndex++) {
      const moves = generateMoveBatch(MOVES_PER_BATCH);
      
      const moveResponse = makeGraphQLRequest(
        playerUrl,
        QUERIES.makeMoves,
        {
          boardId: boardId,
          player: player.username,
          passwordHash: password,
          moves: moves,
        },
        "move"
      );
      
      if (moveResponse) {
        moveOperations.add(MOVES_PER_BATCH);
        console.log(`⚡ [Phase 2 - VU ${vuNum}] Batch ${batchIndex + 1}/${BATCHES_PER_BOARD} on board ${boardIds.indexOf(boardId) + 1}`);
      } else {
        console.error(`❌ [Phase 2 - VU ${vuNum}] Failed move batch ${batchIndex + 1} on board ${boardIds.indexOf(boardId) + 1}`);
        break; // Move to next board if this one fails (probably ended)
      }
      
      sleep(Math.random() * 5 + 10); // 10-15 seconds between batches (like test.ts line 245)
    }
    
    sleep(5); // Wait between boards
  }
  
  console.log(`✅ [Phase 2 - VU ${vuNum}] Completed gameplay session`);
}

// Default export for backwards compatibility (not used in phased approach)
export default function () {
  console.log("Default function should not be called in phased mode");
}
