import http from 'k6/http';
// import { sha256 } from 'k6/crypto';
import { sleep } from 'k6';

const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
const applicationId =
	'95ba53ef3806f843862ee125d79104f25f963ad5fb82aa8d4807a751384a666cc1b7d489a86a8d28ca4c9738052c58f1ebd97acd392f9b5164e189ba5fdb9ee3e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';

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
			startVUs: 20, // This should be outside the stages array
			stages: [
				{ duration: '5m', target: 20 },
				{ duration: '5m', target: 50 },
				{ duration: '120m', target: 50 },
				// { duration: '10m', target: 100 },
				{ duration: '10m', target: 20 }
			]
		}
	}
};

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

			sleep(0.5); // delay between moves
		}

		sleep(4); // Sleep between games
	}
}
