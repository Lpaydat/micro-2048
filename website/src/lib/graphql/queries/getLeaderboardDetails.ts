import type { Client } from '@urql/svelte';
import { gql, queryStore } from '@urql/svelte';

export const GET_LEADERBOARD_DETAILS = gql`
	query GetLeaderboardDetails($leaderboardId: String!) {
		leaderboard(leaderboardId: $leaderboardId) {
			leaderboardId
			name
			host
			rankers {
				username
				score
				boardId
			}
		}
	}
`;

export const getLeaderboardDetails = (client: Client, leaderboardId: string) => {
	return queryStore({ client, query: GET_LEADERBOARD_DETAILS, variables: { leaderboardId } });
};
