import { Client, gql, queryStore } from '@urql/svelte';

export const GET_BOARD_MOVE_HISTORY = gql`
	query GetBoardMoveHistory($boardId: String!, $limit: Int) {
		boardMoveHistory(boardId: $boardId, limit: $limit) {
			boardId
			player
			totalMoves
			moves {
				direction
				timestamp
				boardAfter
				scoreAfter
			}
		}
	}
`;

export const getBoardMoveHistory = (client: Client, boardId: string, limit?: number) => {
	return queryStore({
		client,
		query: GET_BOARD_MOVE_HISTORY,
		variables: { boardId, limit }
	});
};
