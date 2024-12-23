import http from 'k6/http';
// import { sha256 } from 'k6/crypto';
import { sleep } from 'k6';

const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
const applicationId =
	'99822b3483b38c12bb71d00db3a0e1185afb4fbc8d06452d47c771a1bb39a11b79189af31237bb9aa79f94dfba84ab7be1c13f31cafd8d4f4dcd7b94ef0143bbe476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65040000000000000000000000';

const API_URL = `https://u2048.hopto.org/chains/${chainId}/applications/${applicationId}`;

const generateRandomString = (length) => {
	const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
	let result = '';
	for (let i = 0; i < length; i++) {
		result += characters.charAt(Math.floor(Math.random() * characters.length));
	}
	return result;
};

export const options = {
	scenarios: {
		load_test: {
			executor: 'ramping-vus',
			startVUs: 30, // This should be outside the stages array
			stages: [
				{ duration: '15m', target: 30 },
				{ duration: '5m', target: 30 },
				{ duration: '15m', target: 30 },
				// { duration: '5m', target: 40 },
				// { duration: '15m', target: 40 },
				// { duration: '5m', target: 50 },
				// { duration: '15m', target: 50 },
				// { duration: '5m', target: 60 },
				// { duration: '15m', target: 60 },
				{ duration: '10m', target: 20 }
			]
		}
	}
};

// Determine sleep duration based on the target VUs in the current stage
// const stageSleepDurations = {
// 	25: 0.25, // target below 25
// 	30: 0.5, // target 25-30
// 	35: 1, // target 30-35
// 	40: 1.5, // target 35-40
// 	45: 2, // target 40-45
// 	50: 3, // target 45-50
// 	55: 5 // target 50-55
// };

// // Function to get the current stage target based on VUs
// function getCurrentStageTarget() {
// 	const loadTestScenario = options.scenarios.load_test;
// 	if (!loadTestScenario || !loadTestScenario.stages) {
// 		throw new Error('Load test scenario or stages are not defined');
// 	}

// 	for (let i = 0; i < loadTestScenario.stages.length; i++) {
// 		if (__VU <= loadTestScenario.stages[i].target) {
// 			return loadTestScenario.stages[i].target;
// 		}
// 	}
// 	return loadTestScenario.stages[loadTestScenario.stages.length - 1].target;
// }

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
	const newBoardQuery = `mutation newBoard($player: String!, $passwordHash: String!, $timestamp: String!, $seed: String) {
		newBoard(player: $player, passwordHash: $passwordHash, timestamp: $timestamp, seed: $seed)
	}`;

	// Get board
	const getBoardQuery = `query getBoards {
		boards {
			boardId
		}
	}`;

	// Make move query
	const makeMoveQuery = `mutation makeMove($boardId: String!, $direction: Direction!, $player: String!, $timestamp: String!, $passwordHash: String!) {
		makeMove(boardId: $boardId, direction: $direction, player: $player, timestamp: $timestamp, passwordHash: $passwordHash)
	}`;

	const params = {
		headers: {
			'Content-Type': 'application/json'
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
			console.error('Response body is not a string');
		}
	} catch (error) {
		console.error('Failed to parse JSON (getPlayerChainId):', error);
	}
	const DYNAMIC_API_URL = `https://u2048.hopto.org/chains/${playerChainId}/applications/${applicationId}`;

	// First, create 10 boards
	for (let gameCount = 0; gameCount < 10; gameCount++) {
		const timestamp = Date.now().toString();
		const seed = Math.floor(Math.random() * 10_000_000).toString();

		const newBoardVariables = {
			player: username,
			passwordHash,
			timestamp,
			seed
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
			console.error('Response body is not a string');
		}
	} catch (error) {
		console.error('Failed to parse JSON (getBoard):', error);
	}

	// Now play on each board
	for (const boardId of boardIds) {
		// Make 100 moves
		const directions = ['Up', 'Down', 'Left', 'Right'];
		for (let i = 0; i < 100; i++) {
			const makeMoveVariables = {
				boardId,
				direction: directions[i % directions.length],
				player: username,
				timestamp: (Date.now() + i).toString(),
				passwordHash
			};

			http.post(
				DYNAMIC_API_URL,
				JSON.stringify({ query: makeMoveQuery, variables: makeMoveVariables }),
				params
			);

			sleep(1); // delay between moves
		}

		sleep(4); // Sleep between games
	}
}
