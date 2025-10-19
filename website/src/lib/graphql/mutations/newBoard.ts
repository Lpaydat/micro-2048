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
		return null;
	}

	console.log('🎮 New Board Mutation Variables:', {
		player,
		passwordHash: passwordHash.substring(0, 10) + '...',
		timestamp,
		leaderboardId
	});

	const mutation = mutationStore({
		client,
		query: NEW_BOARD,
		variables: { player, passwordHash, timestamp, leaderboardId }
	});

	// Log the mutation result
	mutation.subscribe((result) => {
		console.log('📊 New Board Mutation Result:', result);
		if (result.error) {
			console.error('❌ New Board Mutation Error:', result.error);
		}
	});

	return mutation;
};
