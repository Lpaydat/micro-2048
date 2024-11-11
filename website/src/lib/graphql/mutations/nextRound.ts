import type { Client } from '@urql/svelte';
import { gql, mutationStore } from '@urql/svelte';

const NEXT_ROUND_MUTATION = gql`
	mutation NextRound($gameId: ID!, $player: String!, $timestamp: String!) {
		nextRoundEliminationGame(gameId: $gameId, player: $player, timestamp: $timestamp)
	}
`;

export const nextRound = (client: Client, gameId: string, player: string) => {
	return mutationStore({
		client,
		query: NEXT_ROUND_MUTATION,
		variables: { gameId, player, timestamp: Date.now().toString() }
	});
};
