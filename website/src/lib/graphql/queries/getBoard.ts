import { Client, gql, queryStore } from '@urql/svelte';

export const GET_BOARD = gql`
	query GetBoard {
		board {
			boardId
			player
			chainId
			leaderboardId
			score
			isEnded
			board
			createdAt
		}
	}
`;

export const getBoard = (playerClient: Client) => {
	return queryStore({ client: playerClient, query: GET_BOARD });
};
