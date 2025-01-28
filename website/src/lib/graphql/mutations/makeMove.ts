import type { Client } from '@urql/svelte';
import { gql, mutationStore } from '@urql/svelte';

const MAKE_MOVES = gql`
	mutation MakeMoves($boardId: String!, $moves: String!, $player: String!, $passwordHash: String!) {
		makeMoves(boardId: $boardId, moves: $moves, player: $player, passwordHash: $passwordHash)
	}
`;

export const makeMoves = (client: Client, moves: string, boardId: string) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return;
	}

	mutationStore({
		client,
		query: MAKE_MOVES,
		variables: {
			boardId,
			moves,
			player,
			passwordHash
		}
	});
};
