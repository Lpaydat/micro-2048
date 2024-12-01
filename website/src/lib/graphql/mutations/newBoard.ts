import { Client, gql, mutationStore } from '@urql/svelte';

const NEW_BOARD = gql`
	mutation NewBoard(
		$seed: String!
		$player: String!
		$passwordHash: String!
		$timestamp: String!
		$leaderboardId: String
	) {
		newBoard(
			seed: $seed
			player: $player
			passwordHash: $passwordHash
			timestamp: $timestamp
			leaderboardId: $leaderboardId
		)
	}
`;

export const newGame = (client: Client, seed: string, timestamp: string) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return;
	}

	const leaderboardId = '';

	mutationStore({
		client,
		query: NEW_BOARD,
		variables: { seed, player, passwordHash, timestamp, leaderboardId }
	});
};
