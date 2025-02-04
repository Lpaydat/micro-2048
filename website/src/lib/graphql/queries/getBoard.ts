import type { Client } from '@urql/svelte';
import { gql, queryStore } from '@urql/svelte';

export const GET_BOARD = gql`
	query GetBoard {
		board {
			boardId
			player
			chainId
			leaderboardId
			shardId
			score
			isEnded
			board
			createdAt
		}
	}
`;

export const getBoard = (client: Client) => {
	return queryStore({ client, query: GET_BOARD });
};
