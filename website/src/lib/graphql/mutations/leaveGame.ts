import { Client, gql, mutationStore } from '@urql/svelte';

const LEAVE_GAME = gql`
	mutation LeaveEliminationGame($player: String!, $gameId: String!) {
		leaveEliminationGame(player: $player, gameId: $gameId)
	}
`;

export const leaveGame = (client: Client, player: string, gameId: string) => {
	mutationStore({
		client,
		query: LEAVE_GAME,
		variables: { player, gameId }
	});
};
