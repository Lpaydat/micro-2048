import http from 'k6/http';
import { sha256 } from 'k6/crypto';
import { sleep } from 'k6';

const API_URL =
	'https://u2048.hopto.org/chains/e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65/applications/446ade26b25bb55a0a75aa02b9242ae803969f9ae275c6b787f2b547bf81205f18329514a596fe3921987e9b08e371545085eadb3fd3dcc021d2a1c1b02db5b7e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a650b3000000000000000000000';

const generateRandomString = (length) => {
	const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
	let result = '';
	for (let i = 0; i < length; i++) {
		result += characters.charAt(Math.floor(Math.random() * characters.length));
	}
	return result;
};

const hashSeed = (boardId, username, timestamp) => {
	const input = boardId + username + timestamp;
	const hash = sha256(input, 'hex');
	// Convert first 4 bytes of hex string to number
	return parseInt(hash.substring(0, 8), 16);
};

export const options = {
	scenarios: {
		load_test: {
			executor: 'ramping-vus',
			stages: [
				// Ramp up to 40 VUs over 5 minutes (starting from 10)
				{ duration: '5m', target: 20 },
				// Stay at 40 VUs for 10 minutes
				{ duration: '10m', target: 20 },
				// Ramp down to 20 VUs over 5 minutes
				{ duration: '5m', target: 15 },
				// Stay at 20 VUs for 10 minutes
				{ duration: '10m', target: 20 }
			],
			startVUs: 20 // This should be outside the stages array
		}
	}
};

export default async function () {
	const username = generateRandomString(16);
	const passwordHash = generateRandomString(16);
	const timestamp = Date.now().toString();

	// Register player
	const registerQuery = `mutation registerPlayer($username: String!, $passwordHash: String!) {
		registerPlayer(username: $username, passwordHash: $passwordHash)
	}`;

	const registerVariables = {
		username,
		passwordHash
	};

	// Create new game
	const newBoardQuery = `mutation newBoard($player: String!, $passwordHash: String!, $timestamp: String!, $seed: String) {
		newBoard(player: $player, passwordHash: $passwordHash, timestamp: $timestamp, seed: $seed)
	}`;

	const newBoardVariables = {
		player: username,
		passwordHash,
		timestamp,
		seed: generateRandomString(20) // Random seed
	};

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
	http.post(
		API_URL,
		JSON.stringify({ query: registerQuery, variables: registerVariables }),
		params
	);

	sleep(4);

	// Create new game
	http.post(
		API_URL,
		JSON.stringify({ query: newBoardQuery, variables: newBoardVariables }),
		params
	);

	// // Extract boardId from response
	// const boardId = JSON.parse(newBoardRes.body as string).data.newBoard[0].toString();
	const seed = Math.floor(Math.random() * 10_000_000).toString();
	const boardId = hashSeed(seed, username, timestamp).toString();

	sleep(4);

	// // Make some random moves
	const directions = ['Up', 'Down', 'Left', 'Right'];
	for (let i = 0; i < 5; i++) {
		// Make random moves
		const makeMoveVariables = {
			boardId,
			direction: directions[Math.floor(Math.random() * directions.length)],
			player: username,
			timestamp: (Date.now() + i).toString(), // Increment timestamp for each move
			passwordHash
		};

		http.post(
			API_URL,
			JSON.stringify({ query: makeMoveQuery, variables: makeMoveVariables }),
			params
		);

		sleep(1); // Small delay between moves
	}

	sleep(1);
}
