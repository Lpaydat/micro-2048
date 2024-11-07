import { Client, gql, mutationStore } from '@urql/svelte';

const START_GAME = gql`
	mutation StartEliminationGame($gameId: String!, $player: String!, $timestamp: String!) {
		startEliminationGame(gameId: $gameId, player: $player, timestamp: $timestamp)
	}
`;

export const startGame = (client: Client, gameId: string, player: string) => {
	mutationStore({
		client,
		query: START_GAME,
		variables: { gameId, player, timestamp: Date.now().toString() }
	});
};
