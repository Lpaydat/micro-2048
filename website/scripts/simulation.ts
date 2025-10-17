import http from 'k6/http';
import { check, sleep } from 'k6';

// ========================================
// CONFIGURATION
// ========================================

// Environment configuration - supports both local and production
const ENVIRONMENT = __ENV.ENVIRONMENT || 'production'; // 'local' or 'production'

const CONFIG = {
	local: {
		website: 'localhost',
		port: '8080',
		chainId: '7b9613d4da9ea6adb4399cb61a4fcc831775ce80fa11372219d323463c4ef130',
		applicationId:
			'4e6d771d3d1a21d04038df2250b1fea5e4f061d2108ee552a2ea1c41fdc86aad600f140ec9832578b8343490a00c6573e02f17407429d3bffcad7ae92badf70dc7ca03c01755334323956b5413e24d9a0404b9292fcc91e60700abe80bd5f7a8030000000000000000000000'
	},
	production: {
		website: 'api.micro2048.xyz',
		port: '443',
		chainId: '7b9613d4da9ea6adb4399cb61a4fcc831775ce80fa11372219d323463c4ef130',
		applicationId: '409c67886ae3881c8d03b41c12f27b6cfe3b28c5ea89385aa9eb126155c9a9c9'
	}
};

const config = CONFIG[ENVIRONMENT];
const API_URL = `https://${config.website}:${config.port}/chains/${config.chainId}/applications/${config.applicationId}`;

// ========================================
// SIMULATION PARAMETERS
// ========================================

// Tournament configuration
const TOURNAMENT_ID = __ENV.TOURNAMENT_ID || ''; // Empty = auto-discover

// Bot behavior configuration
const BOT_PERSONALITY = __ENV.BOT_PERSONALITY || 'mixed'; // 'aggressive', 'strategic', 'casual', 'mixed'
const GAMES_PER_BOT = parseInt(__ENV.GAMES_PER_BOT || '3');
const MOVES_PER_GAME = parseInt(__ENV.MOVES_PER_GAME || '50');

// Load testing configuration
export const options = {
	scenarios: {
		simulation: {
			executor: 'ramping-vus',
			startVUs: 1,
			stages: [
				{ target: 10, duration: '30s' }, // Warm up
				{ target: 30, duration: '2m' }, // Ramp up
				{ target: 50, duration: '5m' }, // Main load
				{ target: 30, duration: '2m' }, // Ramp down
				{ target: 10, duration: '1m' }, // Cool down
				{ target: 1, duration: '30s' } // Final cool down
			]
		}
	},
	thresholds: {
		http_req_duration: ['p(95)<2000'], // 95% of requests under 2s
		http_req_failed: ['rate<0.1'] // Error rate under 10%
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

const generateBotName = (personality: string): string => {
	const prefixes = {
		aggressive: ['Speed', 'Flash', 'Turbo', 'Rapid', 'Quick'],
		strategic: ['Think', 'Smart', 'Wise', 'Clever', 'Brain'],
		casual: ['Chill', 'Relax', 'Easy', 'Cool', 'Zen'],
		mixed: ['Bot', 'Player', 'Gamer', 'User', 'AI']
	};

	const prefixList = prefixes[personality as keyof typeof prefixes] || prefixes.mixed;
	const prefix = prefixList[Math.floor(Math.random() * prefixList.length)];
	const suffix = Math.floor(Math.random() * 10000);

	return `${prefix}_${suffix}`;
};

const getPersonalityConfig = (personality: string) => {
	const configs = {
		aggressive: {
			moveDelay: { min: 0.5, max: 2.0 },
			batchSize: 20,
			thinkingTime: 0.1
		},
		strategic: {
			moveDelay: { min: 3.0, max: 8.0 },
			batchSize: 5,
			thinkingTime: 2.0
		},
		casual: {
			moveDelay: { min: 1.0, max: 10.0 },
			batchSize: 10,
			thinkingTime: 1.0
		},
		mixed: {
			moveDelay: { min: 0.5, max: 10.0 },
			batchSize: 10,
			thinkingTime: 1.0
		}
	};

	return configs[personality as keyof typeof configs] || configs.mixed;
};

const generateMoves = (count: number, personality: string): string => {
	const directions: string[] = ['Up', 'Right', 'Down', 'Left'];
	const baseTimestamp = Date.now();

	const moves: [string, string][] = [];
	for (let i = 0; i < count; i++) {
		// Strategic bots prefer certain moves
		let direction: string;
		if (personality === 'strategic') {
			// Prefer Down and Right (generally better in 2048)
			direction =
				Math.random() < 0.6
					? Math.random() < 0.5
						? 'Down'
						: 'Right'
					: directions[Math.floor(Math.random() * directions.length)];
		} else {
			direction = directions[Math.floor(Math.random() * directions.length)];
		}

		const timestamp = baseTimestamp + i * 1000 + Math.random() * 500;
		moves.push([direction, timestamp.toString()]);
	}

	return JSON.stringify(moves);
};

// ========================================
// TOURNAMENT DISCOVERY
// ========================================

const discoverActiveTournaments = () => {
	const query = `
    query getActiveTournaments {
      tournaments {
        tournamentId
        name
        startTime
        endTime
        isActive
      }
    }
  `;

	const response = http.post(API_URL, JSON.stringify({ query }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '30s'
	});

	if (response.status !== 200) {
		console.error('Failed to discover tournaments:', response.status);
		return null;
	}

	try {
		const data = JSON.parse(response.body as string);
		return data.data?.tournaments || [];
	} catch (error) {
		console.error('Failed to parse tournament response:', error);
		return null;
	}
};

const selectBestTournament = (tournaments: any[]) => {
	if (!tournaments || tournaments.length === 0) {
		return null;
	}

	// Filter active tournaments
	const activeTournaments = tournaments.filter((t) => t.isActive);

	if (activeTournaments.length === 0) {
		console.warn('No active tournaments found');
		return null;
	}

	// Select tournament with most players (assumes more activity)
	return activeTournaments.reduce((best, current) => {
		// For now, just return the first active one
		// In a real implementation, you might want more sophisticated selection
		return current;
	});
};

// ========================================
// MAIN SIMULATION LOGIC
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

const createGame = (
	playerChainId: string,
	username: string,
	passwordHash: string,
	leaderboardId: string
) => {
	const playerApiUrl = `https://${config.website}:${config.port}/chains/${playerChainId}/applications/${config.applicationId}`;

	const query = `
    mutation newBoard($player: String!, $passwordHash: String!, $timestamp: String!, $leaderboardId: String!) {
      newBoard(player: $player, passwordHash: $passwordHash, timestamp: $timestamp, leaderboardId: $leaderboardId)
    }
  `;

	const variables = {
		player: username,
		passwordHash,
		timestamp: Date.now().toString(),
		leaderboardId
	};

	return http.post(playerApiUrl, JSON.stringify({ query, variables }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '30s'
	});
};

const getBoards = (playerChainId: string) => {
	const playerApiUrl = `https://${config.website}:${config.port}/chains/${playerChainId}/applications/${config.applicationId}`;

	const query = `
    query getBoards {
      boards {
        boardId
        score
        gameStatus
      }
    }
  `;

	const response = http.post(playerApiUrl, JSON.stringify({ query }), {
		headers: { 'Content-Type': 'application/json' },
		timeout: '30s'
	});

	if (response.status !== 200) {
		return [];
	}

	try {
		const data = JSON.parse(response.body as string);
		return data.data?.boards || [];
	} catch (error) {
		console.error('Failed to parse boards response:', error);
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
	const playerApiUrl = `https://${config.website}:${config.port}/chains/${playerChainId}/applications/${config.applicationId}`;

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
// MAIN SIMULATION FUNCTION
// ========================================

export default function () {
	// Determine bot personality
	const personality =
		BOT_PERSONALITY === 'mixed'
			? ['aggressive', 'strategic', 'casual'][Math.floor(Math.random() * 3)]
			: BOT_PERSONALITY;

	const config = getPersonalityConfig(personality);
	const username = generateBotName(personality);
	const passwordHash = generateRandomString(16);

	console.log(`🤖 Bot ${username} (${personality}) starting simulation...`);

	// Step 1: Register player
	const registerResponse = registerPlayer(username, passwordHash);
	check(registerResponse, {
		'player registration successful': (r) => r.status === 200
	});

	if (registerResponse.status !== 200) {
		console.error(`❌ ${username} failed to register`);
		return;
	}

	sleep(2); // Wait for registration to propagate

	// Step 2: Get player chain ID
	const playerChainId = getPlayerChainId(username);
	if (!playerChainId) {
		console.error(`❌ ${username} failed to get chain ID`);
		return;
	}

	console.log(`✅ ${username} registered with chain ID: ${playerChainId.substring(0, 16)}...`);

	// Step 3: Determine tournament
	let leaderboardId = TOURNAMENT_ID;

	if (!leaderboardId) {
		console.log(`🔍 ${username} discovering tournaments...`);
		const tournaments = discoverActiveTournaments();
		const tournament = selectBestTournament(tournaments);

		if (!tournament) {
			console.error(`❌ ${username} no active tournaments found`);
			return;
		}

		leaderboardId = tournament.tournamentId;
		console.log(`🎯 ${username} selected tournament: ${tournament.name}`);
	}

	// Step 4: Create games
	console.log(`🎮 ${username} creating ${GAMES_PER_BOT} games...`);

	for (let i = 0; i < GAMES_PER_BOT; i++) {
		const createResponse = createGame(playerChainId, username, passwordHash, leaderboardId);
		check(createResponse, {
			[`game ${i + 1} creation successful`]: (r) => r.status === 200
		});

		sleep(1); // Stagger game creation
	}

	sleep(3); // Wait for games to be created

	// Step 5: Get boards and play
	const boards = getBoards(playerChainId);
	console.log(`📋 ${username} found ${boards.length} boards to play`);

	for (const board of boards) {
		if (board.gameStatus === 'Ended') {
			continue; // Skip ended games
		}

		console.log(`🎲 ${username} playing board ${board.boardId.substring(0, 8)}...`);

		// Play moves in batches
		const totalBatches = Math.ceil(MOVES_PER_GAME / config.batchSize);

		for (let batch = 0; batch < totalBatches; batch++) {
			const movesInBatch = Math.min(config.batchSize, MOVES_PER_GAME - batch * config.batchSize);
			const moves = generateMoves(movesInBatch, personality);

			const moveResponse = makeMoves(playerChainId, board.boardId, username, passwordHash, moves);
			check(moveResponse, {
				[`move batch ${batch + 1} successful`]: (r) => r.status === 200
			});

			// Personality-based delay
			const delay =
				config.moveDelay.min + Math.random() * (config.moveDelay.max - config.moveDelay.min);
			sleep(delay);
		}
	}

	console.log(`✅ ${username} completed simulation`);
}
