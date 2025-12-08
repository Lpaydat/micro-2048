import http from 'k6/http';
import { check, sleep } from 'k6';

// ========================================
// CONFIGURATION
// ========================================

// Environment configuration
const ENVIRONMENT = __ENV.ENVIRONMENT || 'production'; // 'local' or 'production'

const CONFIG = {
	local: {
		website: 'localhost',
		port: '8088', // Linera node port (not 5173 which is SvelteKit dev server)
		chainId: '4be348cc444ace2ee56f634333b53aaba008d34371435336838340dfca983524',
		applicationId: 'c570a551f5e9bffa885126ed883f090a8066147356cbfcae0946f324f99deb12'
	},
	production: {
		website: 'api.micro2048.xyz',
		port: '443',
		chainId: '52d89d77d2f103bcf741687984511b4524abb51bcda6716149ea5529e2cbb5b3',
		applicationId: 'f6f39c4f5ed19d9040c968687ea164b127a69146c1da48eed3eaaf22bde6bf0f'
	}
};

// Allow environment variables to override config
const config = {
	website: __ENV.WEBSITE || CONFIG[ENVIRONMENT].website,
	port: __ENV.PORT || CONFIG[ENVIRONMENT].port,
	chainId: __ENV.CHAIN_ID || CONFIG[ENVIRONMENT].chainId,
	applicationId: __ENV.APPLICATION_ID || CONFIG[ENVIRONMENT].applicationId
};

// Use http:// for localhost, https:// for others
const protocol = config.website === 'localhost' ? 'http' : 'https';
// Don't add port for standard HTTPS (443)
const portSuffix = protocol === 'https' && config.port === '443' ? '' : `:${config.port}`;
const API_URL = `${protocol}://${config.website}${portSuffix}/chains/${config.chainId}/applications/${config.applicationId}`;

// ========================================
// TEST PARAMETERS (Adjustable)
// ========================================

const TOURNAMENT_ID =
	__ENV.TOURNAMENT_ID || 'a780876dde256280235ca6937054c914e34b2a33644e713646f55bf0b358d5b6'; // Leaderboard ID
const NUM_PLAYERS = parseInt(__ENV.NUM_PLAYERS || '20'); // Number of mock players
const GAMES_PER_CYCLE = parseInt(__ENV.GAMES_PER_CYCLE || '3'); // Games per player per cycle (not used in infinite mode)
const MOVES_PER_BATCH = parseInt(__ENV.MOVES_PER_BATCH || '15'); // Moves in each batch (10-20 range)
const BATCH_INTERVAL = parseFloat(__ENV.BATCH_INTERVAL || '0.5'); // Seconds between batches (0.5s for fast gameplay)
const BATCHES_PER_GAME = parseInt(__ENV.BATCHES_PER_GAME || '5'); // Number of batches per game
const REGISTRATION_WAIT = parseInt(__ENV.REGISTRATION_WAIT || '10'); // Wait after registration (seconds)
const TEST_DURATION = __ENV.TEST_DURATION || '10m'; // Total test duration (e.g., '10m', '1h')

// ========================================
// K6 OPTIONS
// ========================================

export const options = {
	scenarios: {
		leaderboard_stress: {
			executor: 'constant-vus', // Changed from 'shared-iterations' to run continuously
			vus: NUM_PLAYERS,
			duration: TEST_DURATION // Run for specified duration
		}
	},
	thresholds: {
		http_req_duration: ['p(95)<5000'], // 95% of requests under 5s
		http_req_failed: ['rate<0.2'] // Error rate under 20%
	}
};

// ========================================
// UTILITY FUNCTIONS
// ========================================

const generateRandomString = (length: number): string => {
	const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
	let result = '';
	for (let i = 0; i < length; i++) {
		result += characters.charAt(Math.floor(Math.random() * characters.length));
	}
	return result;
};

const generatePlayerName = (index: number): string => {
	return `StressPlayer_${index}_${Date.now()}`;
};

const generateMoves = (count: number): string => {
	const directions: string[] = ['Up', 'Right', 'Down', 'Left'];
	const baseTimestamp = Date.now();

	const moves: [string, string][] = [];
	for (let i = 0; i < count; i++) {
		const direction = directions[Math.floor(Math.random() * directions.length)];
		// Fast gameplay: 100-200ms per move (average 150ms)
		// Real fast players can make moves every 100-300ms
		const timestamp = Math.floor(baseTimestamp + i * 150 + Math.random() * 100);
		moves.push([direction, timestamp.toString()]);
	}

	return JSON.stringify(moves);
};

// ========================================
// API FUNCTIONS
// ========================================

const registerPlayer = (username: string, passwordHash: string) => {
	const query = `
    mutation registerPlayer($username: String!, $passwordHash: String!) {
      registerPlayer(username: $username, passwordHash: $passwordHash)
    }
  `;

	const variables = { username, passwordHash };

	return http.post(API_URL, JSON.stringify({ query, variables }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '30s'
	});
};

const getPlayerChainId = (username: string) => {
	const query = `
    query getPlayerChainId($username: String!) {
      player(username: $username) {
        chainId
      }
    }
  `;

	const variables = { username };

	const response = http.post(API_URL, JSON.stringify({ query, variables }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '30s'
	});

	if (response.status !== 200) {
		return null;
	}

	try {
		const data = JSON.parse(response.body as string);
		return data.data?.player?.chainId;
	} catch (error) {
		console.error('Failed to parse player chain response:', error);
		return null;
	}
};

const claimChain = (playerChainId: string) => {
	const playerApiUrl = `${protocol}://${config.website}:${config.port}/chains/${playerChainId}/applications/${config.applicationId}`;

	const query = `
    mutation claimChain {
      claimChain
    }
  `;

	return http.post(playerApiUrl, JSON.stringify({ query }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '30s'
	});
};

// Check if player data has propagated to their player chain
const isPlayerReadyOnChain = (playerChainId: string, username: string) => {
	const playerApiUrl = `${protocol}://${config.website}:${config.port}/chains/${playerChainId}/applications/${config.applicationId}`;

	const query = `
    query checkPlayer($username: String!, $passwordHash: String!) {
      checkPlayer(username: $username, passwordHash: $passwordHash)
    }
  `;

	const variables = { username, passwordHash: 'test' };

	const response = http.post(playerApiUrl, JSON.stringify({ query, variables }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '10s'
	});

	if (response.status !== 200) {
		return false;
	}

	try {
		const data = JSON.parse(response.body as string);
		// checkPlayer returns true if player exists and password matches
		return data.data?.checkPlayer === true;
	} catch (error) {
		return false;
	}
};

// Wait for player to be ready on their chain with polling
const waitForPlayerReady = (
	playerChainId: string,
	username: string,
	maxWaitSeconds: number = 60
) => {
	const startTime = Date.now();
	const maxWaitMs = maxWaitSeconds * 1000;

	while (Date.now() - startTime < maxWaitMs) {
		if (isPlayerReadyOnChain(playerChainId, username)) {
			return true;
		}
		console.log(`⏳ [${username}] Waiting for player data to propagate...`);
		sleep(2); // Check every 2 seconds
	}

	return false;
};

const createGame = (
	playerChainId: string,
	username: string,
	passwordHash: string,
	leaderboardId: string
) => {
	const playerApiUrl = `${protocol}://${config.website}:${config.port}/chains/${playerChainId}/applications/${config.applicationId}`;

	const query = `
    mutation newBoard($player: String!, $passwordHash: String!, $timestamp: String!, $leaderboardId: String!) {
      newBoard(player: $player, passwordHash: $passwordHash, timestamp: $timestamp, leaderboardId: $leaderboardId)
    }
  `;

	const timestamp = Date.now().toString();
	const variables = {
		player: username,
		passwordHash,
		timestamp,
		leaderboardId
	};

	console.log(
		`🕐 Creating board with timestamp: ${timestamp}, leaderboardId: ${leaderboardId.substring(0, 16)}...`
	);

	return http.post(playerApiUrl, JSON.stringify({ query, variables }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '30s'
	});
};

const getBoards = (playerChainId: string) => {
	const playerApiUrl = `${protocol}://${config.website}:${config.port}/chains/${playerChainId}/applications/${config.applicationId}`;

	const query = `
    query getBoards {
      boards {
        boardId
        score
        isEnded
      }
    }
  `;

	const response = http.post(playerApiUrl, JSON.stringify({ query }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '30s'
	});

	if (response.status !== 200) {
		console.error(`getBoards failed with status ${response.status}`);
		console.error(`Response: ${response.body}`);
		return [];
	}

	try {
		const data = JSON.parse(response.body as string);
		// Debug: show raw response
		if (!data.data?.boards || data.data.boards.length === 0) {
			console.log(`getBoards response (no boards):`, JSON.stringify(data));
		}
		return data.data?.boards || [];
	} catch (error) {
		console.error('Failed to parse boards response:', error);
		console.error('Raw response:', response.body);
		return [];
	}
};

const makeMoves = (
	playerChainId: string,
	boardId: string,
	username: string,
	passwordHash: string,
	moves: string
) => {
	const playerApiUrl = `${protocol}://${config.website}:${config.port}/chains/${playerChainId}/applications/${config.applicationId}`;

	const query = `
    mutation makeMoves($boardId: String!, $player: String!, $passwordHash: String!, $moves: String!) {
      makeMoves(boardId: $boardId, player: $player, passwordHash: $passwordHash, moves: $moves)
    }
  `;

	const variables = {
		boardId,
		player: username,
		passwordHash,
		moves
	};

	return http.post(playerApiUrl, JSON.stringify({ query, variables }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '60s'
	});
};

// ========================================
// MAIN TEST FUNCTION
// ========================================

export default function () {
	// Validate tournament ID
	if (!TOURNAMENT_ID) {
		console.error('❌ ERROR: TOURNAMENT_ID is required. Set it via environment variable.');
		return;
	}

	// Get VU index for unique player naming
	const vuIndex = __VU;
	const username = generatePlayerName(vuIndex);
	const passwordHash = 'test'; // Use simple password for all mock players

	console.log(`🎯 [Player ${vuIndex}/${NUM_PLAYERS}] Starting leaderboard stress test`);
	console.log(`   Tournament: ${TOURNAMENT_ID.substring(0, 16)}...`);
	console.log(`   Username: ${username}`);
	console.log(`   API URL: ${API_URL.substring(0, 80)}...`);

	// ========================================
	// PHASE 1: REGISTRATION
	// ========================================

	console.log(`📝 [${username}] Registering player...`);
	const registerResponse = registerPlayer(username, passwordHash);
	check(registerResponse, {
		'player registration successful': (r) => r.status === 200
	});

	if (registerResponse.status !== 200) {
		console.error(`❌ [${username}] Registration failed`);
		console.error(`   Status: ${registerResponse.status}`);
		console.error(`   Body: ${registerResponse.body}`);
		return;
	}

	console.log(`✅ [${username}] Registered successfully`);

	// Wait a bit for registration to start propagating
	sleep(5);

	// Get player chain ID
	const playerChainId = getPlayerChainId(username);
	if (!playerChainId) {
		console.error(`❌ [${username}] Failed to get chain ID`);
		return;
	}

	console.log(`✅ [${username}] Got player chain ID: ${playerChainId.substring(0, 16)}...`);

	// 🚀 NEW: Claim the player chain to process inbox messages
	// This triggers block production which processes RegisterPlayer and SubscribeToMainChain messages
	console.log(`⚡ [${username}] Claiming player chain...`);
	const claimResponse = claimChain(playerChainId);
	check(claimResponse, {
		'player chain claim successful': (r) => r.status === 200
	});

	if (claimResponse.status !== 200) {
		console.error(`❌ [${username}] Failed to claim chain`);
		console.error(`   Status: ${claimResponse.status}`);
		console.error(`   Body: ${claimResponse.body}`);
		return;
	}

	console.log(`✅ [${username}] Player chain claimed successfully`);

	// Wait for player data to actually be available on the player chain
	console.log(`⏳ [${username}] Waiting for player data to propagate to player chain...`);
	const isReady = waitForPlayerReady(playerChainId, username, REGISTRATION_WAIT);
	if (!isReady) {
		console.error(`❌ [${username}] Player data did not propagate within ${REGISTRATION_WAIT}s`);
		return;
	}

	console.log(`✅ [${username}] Player is ready on chain!`);

	// ========================================
	// PHASE 2: INFINITE GAME CYCLE LOOP
	// ========================================

	console.log(`🎮 [${username}] Starting infinite game cycle until test duration ends`);
	console.log(`   ${BATCHES_PER_GAME} batches per game, ${MOVES_PER_BATCH} moves per batch`);

	let cycle = 1;
	// Loop infinitely - k6 will stop when duration expires
	while (true) {
		console.log(`\n🔄 [${username}] === CYCLE ${cycle} ===`);

		// Create new board
		console.log(`🆕 [${username}] Creating new board...`);
		const createResponse = createGame(playerChainId, username, passwordHash, TOURNAMENT_ID);
		check(createResponse, {
			[`cycle ${cycle} board creation successful`]: (r) => r.status === 200
		});

		if (createResponse.status !== 200) {
			console.error(`❌ [${username}] Failed to create board in cycle ${cycle}`);
			console.error(`   Response: ${createResponse.body}`);
			continue; // Skip to next cycle
		}

		// Check response body for errors
		try {
			const createData = JSON.parse(createResponse.body as string);
			console.log(`✅ [${username}] Board creation response:`, JSON.stringify(createData));
		} catch (e) {
			console.log(`✅ [${username}] Board creation request sent (status ${createResponse.status})`);
		}

		// Wait longer for board creation to propagate
		console.log(`⏳ [${username}] Waiting for board to be created...`);
		sleep(5);

		// Get all boards for this player
		const boards = getBoards(playerChainId);

		console.log(`📋 [${username}] Found ${boards.length} total boards`);

		if (boards.length === 0) {
			console.error(`❌ [${username}] No boards found after creation in cycle ${cycle}`);
			console.error(`   This might be a timing issue - board creation may need more time`);
			continue;
		}

		// Get the most recently created board (last in array) that isn't ended
		const activeBoards = boards.filter((b: any) => !b.isEnded);

		if (activeBoards.length === 0) {
			console.error(`❌ [${username}] All boards are ended in cycle ${cycle}`);
			continue;
		}

		const latestBoard = activeBoards[activeBoards.length - 1];
		const boardId = latestBoard.boardId;

		console.log(
			`🎲 [${username}] Playing board ${boardId.substring(0, 8)}... (ended: ${latestBoard.isEnded})`
		);
		console.log(`   Full board ID: ${boardId}`);

		// Play batches of moves
		for (let batch = 1; batch <= BATCHES_PER_GAME; batch++) {
			console.log(
				`   📊 [${username}] Batch ${batch}/${BATCHES_PER_GAME}: ${MOVES_PER_BATCH} moves`
			);

			const moves = generateMoves(MOVES_PER_BATCH);
			console.log(`   Moves being sent: ${moves.substring(0, 100)}...`);
			const moveResponse = makeMoves(playerChainId, boardId, username, passwordHash, moves);

			check(moveResponse, {
				[`cycle ${cycle} batch ${batch} successful`]: (r) => r.status === 200
			});

			if (moveResponse.status !== 200) {
				console.error(`❌ [${username}] Failed batch ${batch} in cycle ${cycle}`);
				console.error(`   Status: ${moveResponse.status}`);
				console.error(`   Response: ${moveResponse.body}`);
			} else {
				console.log(`✅ [${username}] Batch ${batch} completed successfully`);
			}

			// Wait between batches (except after last batch)
			if (batch < BATCHES_PER_GAME) {
				console.log(`   ⏳ [${username}] Waiting ${BATCH_INTERVAL}s before next batch...`);
				sleep(BATCH_INTERVAL);
			}
		}

		console.log(`✅ [${username}] Completed cycle ${cycle}`);

		// Increment cycle counter
		cycle++;

		// Small delay before starting next cycle
		sleep(2);
	}

	// Note: This code is unreachable - k6 will stop the test when duration expires
}
