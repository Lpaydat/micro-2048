import { Client, gql, mutationStore } from '@urql/svelte';

// 🚀 IMPROVED: Call updateLeaderboard directly on the leaderboard chain
// This bypasses the cross-chain message issue and triggers aggregation directly
const UPDATE_LEADERBOARD = gql`
	mutation UpdateLeaderboard {
		updateLeaderboard
	}
`;

/**
 * Trigger leaderboard update by calling mutation directly on the leaderboard chain.
 * 
 * @param leaderboardClient - Client configured for the leaderboard chain (leaderboardId IS the chain ID)
 * @returns Mutation store result
 */
export const requestLeaderboardRefresh = (leaderboardClient: Client) => {
	return mutationStore({
		client: leaderboardClient,
		query: UPDATE_LEADERBOARD,
		variables: {}
	});
};
