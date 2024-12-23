import type { Client } from '@urql/svelte';
import { gql, queryStore } from '@urql/svelte';

export const GET_LEADERBOARD_DETAILS = gql`
	query GetLeaderboardDetails {
		leaderboard {
			leaderboardId
			name
			host
			startTime
			endTime
			totalBoards
			totalPlayers
			rankers {
				username
				score
				boardId
			}
		}
	}
`;

export const getLeaderboardDetails = (client: Client) => {
	return queryStore({ client, query: GET_LEADERBOARD_DETAILS });
};
