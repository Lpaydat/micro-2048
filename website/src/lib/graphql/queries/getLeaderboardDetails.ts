import type { Client } from '@urql/svelte';
import { gql, queryStore } from '@urql/svelte';

export const GET_LEADERBOARD_DETAILS = gql`
	query GetLeaderboardDetails($leaderboardId: String, $top: Int, $offset: Int) {
		leaderboard(leaderboardId: $leaderboardId, top: $top, offset: $offset) {
			leaderboardId
			chainId
			name
			description
			isPinned
			host
			startTime
			endTime
			totalBoards
			totalPlayers
			rankers {
				username
				score
				boardId
				isEnded
			}
			shardIds
		}
	}
`;

export const getLeaderboardDetails = (client: Client, leaderboardId?: string, top?: number, offset?: number) => {
	return queryStore({ 
		client, 
		query: GET_LEADERBOARD_DETAILS,
		variables: { leaderboardId, top, offset }
	});
};
