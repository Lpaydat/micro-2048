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
			totalMoves
			moveHistory {
				direction
				timestamp
				boardAfter
				scoreAfter
			}
		}
	}
`;

export const GET_BOARDS = gql`
	query GetBoards($limit: Int) {
		boards(limit: $limit) {
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

export const getBoards = (playerClient: Client, limit: number = 5) => {
	return queryStore({
		client: playerClient,
		query: GET_BOARDS,
		variables: { limit }
	});
};
