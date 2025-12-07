import { Client, gql, mutationStore } from '@urql/svelte';

const REQUEST_LEADERBOARD_REFRESH = gql`
	mutation RequestLeaderboardRefresh(
		$player: String!
		$passwordHash: String!
		$leaderboardId: String!
	) {
		requestLeaderboardRefresh(
			player: $player
			passwordHash: $passwordHash
			leaderboardId: $leaderboardId
		)
	}
`;

export const requestLeaderboardRefresh = (client: Client, leaderboardId: string) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return null;
	}

	console.log('Requesting leaderboard refresh for:', leaderboardId);

	return mutationStore({
		client,
		query: REQUEST_LEADERBOARD_REFRESH,
		variables: { player, passwordHash, leaderboardId }
	});
};
