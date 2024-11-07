import { Client, gql, mutationStore } from '@urql/svelte';

const JOIN_GAME = gql`
	mutation JoinEliminationGame($player: String!, $gameId: String!) {
		joinEliminationGame(player: $player, gameId: $gameId)
	}
`;

export const joinGame = (client: Client, player: string, gameId: string) => {
	mutationStore({
		client,
		query: JOIN_GAME,
		variables: { player, gameId }
	});
};
