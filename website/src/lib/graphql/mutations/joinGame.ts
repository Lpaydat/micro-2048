import { Client, gql, mutationStore } from '@urql/svelte';

const JOIN_GAME = gql`
	mutation JoinEliminationGame($player: String!, $gameId: String!) {
		joinEliminationGame(player: $player, gameId: $gameId)
	}
`;

export const joinGame = (client: Client, player: string, gameId: string) => {
	// user provide username and password
	// we hash password and pass it together with username to backend
	// backend store both of them together
	// field with signature will be `username password_hash`
	// backend will use this signature to verify user
	// backend can still use only username to fetch data
	mutationStore({
		client,
		query: JOIN_GAME,
		variables: { player, gameId }
	});
};
