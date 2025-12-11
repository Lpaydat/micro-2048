import type { Client } from '@urql/svelte';
import { gql, queryStore } from '@urql/svelte';

// Query player's score directly from leaderboard chain (single source of truth)
// Direct key lookup - O(1), no loop needed
// Must be called with leaderboard chain client
export const GET_PLAYER_BEST_SCORE = gql`
	query GetPlayerBestScore($player: String!) {
		playerBestScore(player: $player)
	}
`;

export const getPlayerBestScore = (client: Client, player: string) => {
	return queryStore({
		client,
		query: GET_PLAYER_BEST_SCORE,
		variables: { player }
	});
};
