import { Client, gql, mutationStore } from '@urql/svelte';

// Define the MultiplayerGameAction enum
enum MultiplayerGameAction {
	Start = 'Start',
	End = 'End',
	Trigger = 'Trigger',
	Join = 'Join',
	Leave = 'Leave',
	NextRound = 'NextRound'
}

const ELIMINATION_GAME_ACTION = gql`
	mutation EliminationGameAction(
		$gameId: String!
		$action: MultiplayerGameAction!
		$player: String!
		$passwordHash: String!
		$timestamp: String!
	) {
		eliminationGameAction(
			gameId: $gameId
			action: $action
			player: $player
			passwordHash: $passwordHash
			timestamp: $timestamp
		)
	}
`;

const mutation = (client: Client, gameId: string, action: MultiplayerGameAction) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return;
	}

	mutationStore({
		client,
		query: ELIMINATION_GAME_ACTION,
		variables: {
			gameId,
			action,
			player,
			passwordHash,
			timestamp: Date.now().toString()
		}
	});
};

export const joinGame = (client: Client, gameId: string) => {
	mutation(client, gameId, MultiplayerGameAction.Join);
};

export const leaveGame = (client: Client, gameId: string) => {
	mutation(client, gameId, MultiplayerGameAction.Leave);
};

export const endGame = (client: Client, gameId: string) => {
	mutation(client, gameId, MultiplayerGameAction.End);
};

export const triggerGame = (client: Client, gameId: string) => {
	mutation(client, gameId, MultiplayerGameAction.Trigger);
};

export const nextRound = (client: Client, gameId: string) => {
	mutation(client, gameId, MultiplayerGameAction.NextRound);
};

export const startGame = (client: Client, gameId: string) => {
	mutation(client, gameId, MultiplayerGameAction.Start);
};
