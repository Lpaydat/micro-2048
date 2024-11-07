import { Client, gql, mutationStore } from '@urql/svelte';

const END_GAME = gql`
	mutation EndEliminationGame($gameId: String!, $player: String!, $timestamp: String!) {
		endEliminationGame(gameId: $gameId, player: $player, timestamp: $timestamp)
	}
`;

export const endGame = (client: Client, gameId: string, player: string) => {
	mutationStore({
		client,
		query: END_GAME,
		variables: { gameId, player, timestamp: Date.now().toString() }
	});
};
