import { Client, gql, queryStore } from '@urql/svelte';

export const GET_BOARD_PAGINATED = gql`
	query GetBoardPaginated($boardId: String, $moveOffset: Int, $moveLimit: Int) {
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

export const getBoardPaginated = (
	client: Client,
	boardId?: string,
	moveOffset?: number,
	moveLimit?: number
) => {
	return queryStore({
		client,
		query: GET_BOARD_PAGINATED,
		variables: {
			boardId,
			moveOffset,
			moveLimit
		},
		requestPolicy: 'network-only' // Always fetch fresh data for pagination
	});
};
