import type { Client } from '@urql/svelte';
import { gql, mutationStore } from '@urql/svelte';

const MAKE_MOVE = gql`
	mutation MakeMove(
		$boardId: ID!
		$direction: String!
		$player: String!
		$timestamp: String!
		$passwordHash: String!
	) {
		makeMove(
			boardId: $boardId
			direction: $direction
			player: $player
			timestamp: $timestamp
			passwordHash: $passwordHash
		)
	}
`;

// Enum for move directions
const directionList = {
	Up: 'Up',
	Down: 'Down',
	Left: 'Left',
	Right: 'Right'
};

export const makeMove = (
	client: Client,
	timestamp: string,
	boardId: string,
	direction?: string
) => {
	const formattedDirection = direction?.replace('Arrow', '');
	if (formattedDirection && !Object.values(directionList).includes(formattedDirection)) {
		console.error('Invalid direction:', direction);
		return;
	}

	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return;
	}

	mutationStore({
		client,
		query: MAKE_MOVE,
		variables: {
			boardId,
			direction: formattedDirection,
			player,
			timestamp,
			passwordHash
		}
	});
};
