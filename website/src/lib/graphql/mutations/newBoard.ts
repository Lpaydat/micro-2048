import { Client, gql, mutationStore } from '@urql/svelte';

const NEW_BOARD = gql`
	mutation NewBoard(
		$player: String!
		$passwordHash: String!
		$timestamp: String!
		$leaderboardId: String!
		$rhythmTrackIndex: Int
	) {
		newBoard(
			player: $player
			passwordHash: $passwordHash
			timestamp: $timestamp
			leaderboardId: $leaderboardId
			rhythmTrackIndex: $rhythmTrackIndex
		)
	}
`;

/**
 * Create a new game board
 * @param client - GraphQL client
 * @param timestamp - Current timestamp in milliseconds
 * @param leaderboardId - Tournament/leaderboard ID
 * @param rhythmTrackIndex - 🎵 Rhythm mode: which music track to use
 *   - undefined/null = no rhythm mode (-1 stored in contract)
 *   - 0+ = specific track index
 */
export const newGame = (
	client: Client,
	timestamp: string,
	leaderboardId: string,
	rhythmTrackIndex?: number
) => {
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
		leaderboardId,
		rhythmTrackIndex: rhythmTrackIndex ?? 'none (no rhythm)'
	});

	const mutation = mutationStore({
		client,
		query: NEW_BOARD,
		variables: {
			player,
			passwordHash,
			timestamp,
			leaderboardId,
			rhythmTrackIndex: rhythmTrackIndex ?? null
		}
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
