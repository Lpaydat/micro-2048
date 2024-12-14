import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
	stages: [
		{ duration: '30s', target: 50 }, // Ramp up to 50 users over 30 seconds
		{ duration: '1m', target: 50 }, // Stay at 50 users for 1 minute
		{ duration: '30s', target: 0 } // Ramp down to 0 users over 30 seconds
	]
};

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

export default function () {
	const username = generateRandomString(8); // Random username
	const passwordHash = generateRandomString(16); // Random password hash

	const query = `mutation registerPlayer($username: String!, $passwordHash: String!) {
    registerPlayer(username: $username, passwordHash: $passwordHash)
  }`;

	const variables = {
		username: username,
		passwordHash: passwordHash
	};

	const payload = JSON.stringify({ query, variables });

	const params = {
		headers: {
			'Content-Type': 'application/json'
		}
	};

	const res = http.post(API_URL, payload, params);

	check(res, {
		'status is 200': (r) => r.status === 200,
		'response has data': (r) => JSON.parse(r.body as string).data !== null
	});

	sleep(1);
}
