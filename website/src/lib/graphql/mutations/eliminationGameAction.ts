import { userStore } from '$lib/stores/userStore';
import type { Client } from '@urql/svelte';
import { gql, mutationStore } from '@urql/svelte';
import { get } from 'svelte/store';

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
		$action: MultiplayerGameAction!
		$player: String!
		$passwordHash: String!
		$requesterChainId: String!
		$timestamp: String!
	) {
		eliminationGameAction(
			action: $action
			player: $player
			passwordHash: $passwordHash
			requesterChainId: $requesterChainId
			timestamp: $timestamp
		)
	}
`;

const mutation = (client: Client, action: MultiplayerGameAction) => {
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
			action,
			player,
			passwordHash,
			timestamp: Date.now().toString(),
			requesterChainId: get(userStore).chainId
		}
	});
};

export const joinGame = (client: Client) => {
	mutation(client, MultiplayerGameAction.Join);
};

export const leaveGame = (client: Client) => {
	mutation(client, MultiplayerGameAction.Leave);
};

export const endGame = (client: Client) => {
	mutation(client, MultiplayerGameAction.End);
};

export const triggerGame = (client: Client) => {
	mutation(client, MultiplayerGameAction.Trigger);
};

export const nextRound = (client: Client) => {
	mutation(client, MultiplayerGameAction.NextRound);
};

export const startGame = (client: Client) => {
	mutation(client, MultiplayerGameAction.Start);
};
