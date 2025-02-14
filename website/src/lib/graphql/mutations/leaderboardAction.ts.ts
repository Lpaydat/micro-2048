import { Client, gql, mutationStore } from '@urql/svelte';

enum LeaderboardAction {
	Create = 'Create',
	Update = 'Update',
	Delete = 'Delete',
	TogglePin = 'TogglePin'
}

export const LEADERBOARD_ACTION = gql`
	mutation LeaderboardAction(
		$leaderboardId: String!
		$action: LeaderboardAction!
		$settings: LeaderboardSettings!
		$player: String!
		$passwordHash: String!
		$timestamp: String!
	) {
		leaderboardAction(
			action: $action
			leaderboardId: $leaderboardId
			player: $player
			passwordHash: $passwordHash
			timestamp: $timestamp
			settings: $settings
		)
	}
`;

export type LeaderboardSettings = {
	id?: string;
	name: string;
	description?: string;
	startTime: string;
	endTime: string;
	isPinned?: boolean;
};

const mutation = (
	client: Client,
	leaderboardId: string,
	action: LeaderboardAction,
	settings: LeaderboardSettings
) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return;
	}

	const timestamp = Date.now().toString();

	mutationStore({
		client,
		query: LEADERBOARD_ACTION,
		variables: { leaderboardId, action, player, passwordHash, settings, timestamp }
	});
};

export const createLeaderboard = (client: Client, settings: LeaderboardSettings) => {
	mutation(client, '', LeaderboardAction.Create, settings);
};

export const updateLeaderboard = (
	client: Client,
	leaderboardId: string,
	settings: LeaderboardSettings
) => {
	mutation(client, leaderboardId, LeaderboardAction.Update, settings);
};

export const deleteLeaderboard = (client: Client, leaderboardId: string) => {
	mutation(client, leaderboardId, LeaderboardAction.Delete, {
		name: '',
		startTime: '',
		endTime: ''
	});
};

export const togglePinLeaderboard = (client: Client, leaderboardId: string) => {
	mutation(client, leaderboardId, LeaderboardAction.TogglePin, {
		name: '',
		startTime: '0',
		endTime: '0'
	});
};
