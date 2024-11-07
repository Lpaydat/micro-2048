import { Client, gql, mutationStore } from '@urql/svelte';

const MAKE_MOVE = gql`
	mutation MakeMove($boardId: ID!, $direction: String!) {
		makeMove(boardId: $boardId, direction: $direction)
	}
`;

// Enum for move directions
const directionList = {
	Up: 'Up',
	Down: 'Down',
	Left: 'Left',
	Right: 'Right'
};

export const makeMove = (client: Client, boardId: string, direction: string) => {
	const formattedDirection = direction.replace('Arrow', '');
	if (!Object.values(directionList).includes(formattedDirection)) {
		console.error('Invalid direction:', direction);
		return;
	}

	mutationStore({
		client,
		query: MAKE_MOVE,
		variables: { boardId, direction: formattedDirection }
	});
};
