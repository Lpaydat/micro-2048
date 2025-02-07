import http from 'k6/http';
// import { sha256 } from 'k6/crypto';
import { sleep } from 'k6';

const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
const applicationId =
	'ca38e7926e6ca7bd3d23582021f7c9a5f70faa4e41f14363de02be2e4e3c02deee164168864bf14def3c9951cd6994011e6addfbb63d37f00c8c1c28e3cedcb8e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';

const API_URL = `http://localhost:8080/chains/${chainId}/applications/${applicationId}`;

const generateRandomString = (length) => {
	const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
	let result = '';
	for (let i = 0; i < length; i++) {
		result += characters.charAt(Math.floor(Math.random() * characters.length));
	}
	return result;
};

// Update options with proper thresholds and system resource management
export const options = {
	discardResponseBodies: true,
	noConnectionReuse: false, // Enable connection reuse
	systemTags: ['status', 'error', 'check', 'method', 'url'],
	thresholds: {
		http_req_failed: ['rate<0.01'], // 1% errors allowed
		http_req_duration: ['p(95)<5000']
	},
	maxRedirects: 4,
	scenarios: {
		load_test: {
			executor: 'ramping-arrival-rate',
			startRate: 50,
			timeUnit: '1s',
			preAllocatedVUs: 50,
			maxVUs: 200,
			stages: [
				{ target: 50, duration: '10m' },
				{ target: 60, duration: '2m' },
				{ target: 60, duration: '10m' },
				{ target: 70, duration: '2m' },
				{ target: 70, duration: '10m' },
				{ target: 80, duration: '2m' },
				{ target: 80, duration: '10m' },
				{ target: 20, duration: '2m' },
				{ target: 20, duration: '10m' }
			]
		}
	}
};

// Improved move simulation with batch operations
const generateMoveBatch = (count) => {
	const directions = ['Up', 'Right', 'Down', 'Left'];
	const baseTimestamp = Date.now();

	return JSON.stringify(
		Array.from({ length: count }, (_, i) => [
			directions[Math.floor(Math.random() * directions.length)],
			(baseTimestamp + i).toString()
		])
	);
};

// Simplified test flow with better error handling
export default async function () {
	const username = generateRandomString(16);
	const passwordHash = generateRandomString(16);

	// Register player
	const registerQuery = `mutation registerPlayer($username: String!, $passwordHash: String!) {
		registerPlayer(username: $username, passwordHash: $passwordHash)
	}`;

	// Get Player Chain ID
	const getPlayerChainIdQuery = `query getPlayerChainId($username: String!) {
		player(username: $username) {
			chainId
		}
	}`;

	// Create new game
	const newBoardQuery = `mutation newBoard(
		$player: String!,
		$passwordHash: String!,
		$timestamp: String!,
		$seed: String,
		$leaderboardId: String!,
		$shardId: String!
	) {
		newBoard(
			player: $player,
			passwordHash: $passwordHash,
			timestamp: $timestamp,
			seed: $seed,
			leaderboardId: $leaderboardId,
			shardId: $shardId
		)
	}`;

	// Get board
	const getBoardQuery = `query getBoards {
		boards {
			boardId
		}
	}`;

	// Make move query
	const makeMoveQuery = `mutation makeMove(
						$boardId: String!, 
						$player: String!, 
						$passwordHash: String!,
						$moves: String!
					) {
						makeMoves(
							boardId: $boardId
							player: $player
							passwordHash: $passwordHash
							moves: $moves
						)
					}`;

	const params = {
		timeout: '120s',
		headers: {
			'Content-Type': 'application/json'
			// Connection: 'keep-alive'
		}
		// responseType: 'none'
	};

	// Execute registration
	const registerVariables = {
		username,
		passwordHash
	};

	http.post(
		API_URL,
		JSON.stringify({ query: registerQuery, variables: registerVariables }),
		params
	);

	sleep(5);

	const getPlayerChainIdRes = http.post(
		API_URL,
		JSON.stringify({ query: getPlayerChainIdQuery, variables: { username } }),
		params
	);

	// Get player chain ID
	let resJson;
	let playerChainId;
	try {
		if (typeof getPlayerChainIdRes.body === 'string') {
			resJson = JSON.parse(getPlayerChainIdRes.body);
			playerChainId = resJson.data.player.chainId;
		} else {
			console.error('Response body is not a string (getPlayerChainId):', getPlayerChainIdRes.body);
		}
	} catch (error) {
		console.error('Failed to parse JSON (getPlayerChainId):', error);
	}
	const DYNAMIC_API_URL = `http://localhost:8080/chains/${playerChainId}/applications/${applicationId}`;

	const leaderboardId = 'ae41b40b288a1e7ed064e2ff749a9ce3e780a5742dca074e6015e77e9dd373f8';
	const shards = [
		'538487ab8fab9220b625c514caa2a7c87cd7d99da380e724285bf9969cb108a5',
		'5c531fb806cee764e2effbdb9787159e146d0cc982b72b993babae20c0716c27',
		'ba9be87de439e1f7c676fb0bf1454db229ff2b4aa870f617a2969d7e4aa33fa8',
		'ac8e31bb8255fa66148c1d52ba22fbbf04bcc38917fa1bc09658d77ca5e1c216',
		'a940a388ad7f223e45727402e70db50da997e75e983053147fabdc7a98c9c827',
		'67140c3e159367012fd7a13dc06fdaec7530e4392f36ebdbd3c66ab807bf4be2',
		'c65773cb1c58497ab0e651b885ab09ba16bf96164b82d61538998ed47ab2e3cb',
		'8b619595278f0a75e122b3b0f380e8f08bcbf7cfdee1b71ff152dafba1486434'
	];
	const getShardId = () => {
		const randomIndex = Math.floor(Math.random() * shards.length);
		return shards[randomIndex];
	};

	// First, create 5 boards
	for (let gameCount = 0; gameCount < 5; gameCount++) {
		const timestamp = Date.now().toString();
		const seed = Math.floor(Math.random() * 10_000_000).toString();

		const newBoardVariables = {
			player: username,
			passwordHash,
			timestamp,
			seed,
			leaderboardId,
			shardId: getShardId()
		};

		// Create new game
		http.post(
			DYNAMIC_API_URL,
			JSON.stringify({ query: newBoardQuery, variables: newBoardVariables }),
			params
		);

		sleep(2);
	}

	// Get all board IDs
	const getBoardRes = http.post(DYNAMIC_API_URL, JSON.stringify({ query: getBoardQuery }), params);

	let boardIds = [];
	try {
		if (typeof getBoardRes.body === 'string') {
			const resJson = JSON.parse(getBoardRes.body);
			boardIds = resJson.data.boards.map((board) => board.boardId);
		} else {
			console.error('Response body is not a string (getBoard):', getBoardRes.body);
		}
	} catch (error) {
		console.error('Failed to parse JSON (getBoard):', error);
	}

	// Batch move operations (10 moves per request)
	const BATCH_SIZE = 10;
	const TOTAL_MOVES = 50;

	for (const boardId of boardIds) {
		for (let batchIndex = 0; batchIndex < TOTAL_MOVES / BATCH_SIZE; batchIndex++) {
			const moves = generateMoveBatch(BATCH_SIZE);

			const makeMoveVariables = {
				boardId,
				player: username,
				passwordHash,
				moves
			};

			const res = http.post(
				DYNAMIC_API_URL,
				JSON.stringify({
					query: makeMoveQuery,
					variables: makeMoveVariables
				}),
				params
			);

			// Add basic error handling
			if (res.status !== 200) {
				console.error(`Move batch failed: ${res.status_text}`);
			}

			// Add forced GC and delay
			if (batchIndex % 5 === 0) {
				if (typeof globalThis.Bun !== 'undefined') globalThis.Bun.gc(true);
				sleep(1); // Extra breathing room
			}

			sleep(Math.random() * 2 + 1);
		}

		sleep(10); // Reduced sleep between games
	}
}
