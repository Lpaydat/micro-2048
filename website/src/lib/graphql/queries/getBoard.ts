import { Client, gql, queryStore } from '@urql/svelte';

export const GET_BOARD = gql`
	query GetBoard($boardId: String, $moveOffset: Int, $moveLimit: Int) {
		board(boardId: $boardId, moveOffset: $moveOffset, moveLimit: $moveLimit) {
			boardId
			player
			chainId
			leaderboardId
			score
			isEnded
			board
			createdAt
			totalMoves
			moveOffset
			moveLimit
			hasMoreMoves
			rhythmTrackIndex
			moveHistory {
				direction
				timestamp
				boardAfter
				scoreAfter
				beatNumber
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

export const getBoard = (
	playerClient: Client,
	boardId?: string,
	moveOffset?: number,
	moveLimit?: number
) => {
	return queryStore({
		client: playerClient,
		query: GET_BOARD,
		variables: {
			boardId,
			moveOffset,
			moveLimit
		}
	});
};

export const getBoards = (playerClient: Client, limit: number = 5) => {
	return queryStore({
		client: playerClient,
		query: GET_BOARDS,
		variables: { limit }
	});
};
