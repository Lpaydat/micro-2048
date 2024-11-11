import { Client, gql, mutationStore } from '@urql/svelte';

const TRIGGER_GAME_EVENT = gql`
	mutation TriggerGameEvent($gameId: String!, $player: String!, $timestamp: String!) {
		triggerEliminationGame(gameId: $gameId, player: $player, timestamp: $timestamp)
	}
`;

export const triggerGameEvent = (client: Client, gameId: string, player: string) => {
	return mutationStore({
		client,
		query: TRIGGER_GAME_EVENT,
		variables: { gameId, player, timestamp: Date.now().toString() }
	});
};
