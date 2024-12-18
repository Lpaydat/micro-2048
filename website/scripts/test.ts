import http from 'k6/http';
// import { sha256 } from 'k6/crypto';
import { sleep } from 'k6';

const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
const applicationId =
	'9a61b2c4b42d67188fddb199ce3d573a76d36deb72d40da0b6e91ee04fe960b86dfc7b6e2d8acd1dd137ca78dc7d0206f224f6727126471a9acda9f56f34f70ae476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';

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
			startVUs: 10, // This should be outside the stages array
			stages: [
				{ duration: '5m', target: 20 },
				{ duration: '10m', target: 50 },
				{ duration: '10m', target: 100 },
				{ duration: '10m', target: 50 }
			]
		}
	}
};

export default async function () {
	const username = generateRandomString(16);
	const passwordHash = generateRandomString(16);
	const timestamp = Date.now().toString();
	const seed = Math.floor(Math.random() * 10_000_000).toString();

	// Register player
	const registerQuery = `mutation registerPlayer($username: String!, $passwordHash: String!) {
		registerPlayer(username: $username, passwordHash: $passwordHash)
	}`;

	const registerVariables = {
		username,
		passwordHash
	};

	// Get Player Chain ID
	const getPlayerChainIdQuery = `query getPlayerChainId($username: String!) {
		player(username: $username) {
			chainId
		}
	}`;

	const getPlayerChainIdVariables = {
		username
	};

	// Create new game
	const newBoardQuery = `mutation newBoard($player: String!, $passwordHash: String!, $timestamp: String!, $seed: String) {
		newBoard(player: $player, passwordHash: $passwordHash, timestamp: $timestamp, seed: $seed)
	}`;

	const newBoardVariables = {
		player: username,
		passwordHash,
		timestamp,
		seed
	};

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
	http.post(
		API_URL,
		JSON.stringify({ query: registerQuery, variables: registerVariables }),
		params
	);

	sleep(5);

	const getPlayerChainIdRes = http.post(
		API_URL,
		JSON.stringify({ query: getPlayerChainIdQuery, variables: getPlayerChainIdVariables }),
		params
	);
	// Check if the response body is not empty and is valid JSON
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

	// Create new game
	http.post(
		DYNAMIC_API_URL,
		JSON.stringify({ query: newBoardQuery, variables: newBoardVariables }),
		params
	);

	sleep(2);

	const getBoardRes = http.post(DYNAMIC_API_URL, JSON.stringify({ query: getBoardQuery }), params);

	let boardId;
	try {
		if (typeof getBoardRes.body === 'string') {
			resJson = JSON.parse(getBoardRes.body);
			boardId = resJson.data.boards[0].boardId;
		} else {
			console.error('Response body is not a string');
		}
	} catch (error) {
		console.error('Failed to parse JSON (getBoard):', error);
	}

	// // Make some random moves
	const directions = ['Up', 'Down', 'Left', 'Right'];
	for (let i = 0; i < 100; i++) {
		// Make random moves
		const makeMoveVariables = {
			boardId,
			direction: directions[i % directions.length],
			player: username,
			timestamp: (Date.now() + i).toString(), // Increment timestamp for each move
			passwordHash
		};

		http.post(
			DYNAMIC_API_URL,
			JSON.stringify({ query: makeMoveQuery, variables: makeMoveVariables }),
			params
		);

		sleep(0.1); // Small delay between moves
	}

	sleep(1);
}
