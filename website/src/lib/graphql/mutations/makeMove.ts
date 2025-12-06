import type { Client, OperationResult } from '@urql/svelte';
import { gql } from '@urql/svelte';

const MAKE_MOVES = gql`
	mutation MakeMoves($boardId: String!, $moves: String!, $player: String!, $passwordHash: String!) {
		makeMoves(boardId: $boardId, moves: $moves, player: $player, passwordHash: $passwordHash)
	}
`;

export interface MakeMoveResult {
	success: boolean;
	error?: string;
}

/**
 * Submit moves to the backend with proper error handling.
 * Returns a promise that resolves when the mutation completes.
 */
export const makeMoves = async (
	client: Client,
	moves: string,
	boardId: string
): Promise<MakeMoveResult> => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return { success: false, error: 'Player or password hash not found' };
	}

	try {
		const result: OperationResult = await client
			.mutation(MAKE_MOVES, {
				boardId,
				moves,
				player,
				passwordHash
			})
			.toPromise();

		if (result.error) {
			console.error('makeMoves mutation error:', result.error);
			return { success: false, error: result.error.message };
		}

		return { success: true };
	} catch (error: unknown) {
		const errorMessage = error instanceof Error ? error.message : 'Unknown error';
		console.error('makeMoves exception:', errorMessage);
		return { success: false, error: errorMessage };
	}
};

/**
 * Fire-and-forget version for non-critical syncs (legacy compatibility).
 * Use makeMoves() for critical operations where you need confirmation.
 */
export const makeMovesFireAndForget = (client: Client, moves: string, boardId: string): void => {
	makeMoves(client, moves, boardId).catch((err) => {
		console.error('makeMovesFireAndForget failed:', err);
	});
};
