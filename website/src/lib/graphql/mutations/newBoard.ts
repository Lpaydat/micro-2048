import { Client, gql, mutationStore } from '@urql/svelte';

const NEW_BOARD = gql`
	mutation NewBoard(
		$player: String!
		$passwordHash: String!
		$playerChainId: String!
		$timestamp: String!
	) {
		newBoard(
			player: $player
			passwordHash: $passwordHash
			playerChainId: $playerChainId
			timestamp: $timestamp
		)
	}
`;

export const newGame = (client: Client, timestamp: string) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');
	const playerChainId = localStorage.getItem('chainId');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return;
	}

	mutationStore({
		client,
		query: NEW_BOARD,
		variables: { player, passwordHash, playerChainId, timestamp }
	});
};
