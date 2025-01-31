import http from 'k6/http';
// import { sha256 } from 'k6/crypto';
import { sleep } from 'k6';

const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
const applicationId =
	'20ae69cbf63624a0e0956520078ae8a9ba36ec8f0af1a385c786e73a0a4f7d2b307bdf87f149381bb26c4d51d09af191c19ad7c54816be326eada7272df05146e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';
const website = 'u2048.hopto.org';

const API_URL = `https://${website}/chains/${chainId}/applications/${applicationId}`;

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
	scenarios: {
		load_test: {
			executor: 'ramping-vus',
			startVUs: 20,
			stages: [
				{ target: 30, duration: '2m' }, // Initial gentle warmup
				{ target: 50, duration: '3m' }, // Ramp up to moderate load
				{ target: 50, duration: '5m' }, // Maintain moderate load to establish baseline
				// { target: 100, duration: '5m' }, // Ramp up to medium load
				// { target: 100, duration: '10m' }, // Sustained medium load test
				// { target: 150, duration: '5m' }, // Ramp up to high load
				// { target: 150, duration: '10m' }, // Sustained high load test
				// { target: 200, duration: '5m' }, // Ramp up to peak load
				// { target: 200, duration: '10m' }, // Peak load stress test
				// { target: 100, duration: '5m' }, // Gradual ramp down
				{ target: 50, duration: '3m' }, // Further ramp down
				{ target: 20, duration: '2m' } // Cool down to baseline
			]
		}
	},
	// Add batch support and proper timeouts
	batch: 20, // Group similar requests
	batchPerHost: 10 // Prevent host overload
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
	const newBoardQuery = `mutation newBoard($player: String!, $passwordHash: String!, $timestamp: String!, $seed: String, $leaderboardId: String!, $shardId: String!) {
		newBoard(player: $player, passwordHash: $passwordHash, timestamp: $timestamp, seed: $seed, leaderboardId: $leaderboardId, shardId: $shardId)
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
			'Content-Type': 'application/json',
			Connection: 'keep-alive'
		}
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

	// Update response handling with proper type checking
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
			console.error('Missing chain ID in response:', JSON.stringify(resJson));
		}
	} catch (error) {
		console.error('Failed to parse JSON (getPlayerChainId):', error);
	}
	const DYNAMIC_API_URL = `https://${website}/chains/${playerChainId}/applications/${applicationId}`;

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
	if (getBoardRes.body) {
		try {
			const body =
				typeof getBoardRes.body === 'string'
					? getBoardRes.body
					: new TextDecoder().decode(getBoardRes.body);

			const resJson = JSON.parse(body);

			if (Array.isArray(resJson?.data?.boards)) {
				boardIds = resJson.data.boards.map((board) => board.boardId);
			} else {
				console.error('Unexpected board response structure:', JSON.stringify(resJson));
			}
		} catch (error) {
			console.error('Failed to parse board response:', error);
		}
	}

	// Batch move operations (10 moves per request)
	const BATCH_SIZE = 800;
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

			sleep(Math.random() * 10 + 25);
		}

		sleep(10); // Reduced sleep between games
	}
}
