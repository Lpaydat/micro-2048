import { Client, gql, mutationStore } from '@urql/svelte';

const NEW_BOARD = gql`
	mutation NewBoard(
		$player: String!
		$passwordHash: String!
		$timestamp: String!
		$leaderboardId: String!
	) {
		newBoard(
			player: $player
			passwordHash: $passwordHash
			timestamp: $timestamp
			leaderboardId: $leaderboardId
		)
	}
`;

export const newGame = (client: Client, timestamp: string, leaderboardId: string) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return;
	}

	mutationStore({
		client,
		query: NEW_BOARD,
		variables: { player, passwordHash, timestamp, leaderboardId }
	});
};
