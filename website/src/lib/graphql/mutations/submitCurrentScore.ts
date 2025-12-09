import { Client, gql, mutationStore } from '@urql/svelte';

const SUBMIT_CURRENT_SCORE = gql`
	mutation SubmitCurrentScore($boardId: String!, $player: String!, $passwordHash: String!) {
		submitCurrentScore(boardId: $boardId, player: $player, passwordHash: $passwordHash)
	}
`;

/**
 * Submit current board score to leaderboard manually.
 * Only sends if: score > 0 AND score > player's tournament best
 * 
 * @param playerClient - Client configured for the player's chain
 * @param boardId - The board ID to submit score for
 * @param player - The player's username
 * @param passwordHash - The player's password hash
 * @returns Mutation store result
 */
export const submitCurrentScore = (
	playerClient: Client,
	boardId: string,
	player: string,
	passwordHash: string
) => {
	console.log('Submitting current score for board:', boardId);

	return mutationStore({
		client: playerClient,
		query: SUBMIT_CURRENT_SCORE,
		variables: {
			boardId,
			player,
			passwordHash
		}
	});
};
