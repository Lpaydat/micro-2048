import { Client, gql, mutationStore } from '@urql/svelte';

enum EventLeaderboardAction {
	Create = 'Create',
	Update = 'Update',
	Delete = 'Delete',
	TogglePin = 'TogglePin'
}

export const EVENT_LEADERBOARD_ACTION = gql`
	mutation EventLeaderboardAction(
		$leaderboardId: String!
		$action: EventLeaderboardAction!
		$settings: EventLeaderboardSettings!
		$player: String!
		$passwordHash: String!
		$timestamp: String!
	) {
		eventLeaderboardAction(
			action: $action
			leaderboardId: $leaderboardId
			player: $player
			passwordHash: $passwordHash
			timestamp: $timestamp
			settings: $settings
		)
	}
`;

export type EventSettings = {
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
	action: EventLeaderboardAction,
	settings: EventSettings
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
		query: EVENT_LEADERBOARD_ACTION,
		variables: { leaderboardId, action, player, passwordHash, settings, timestamp }
	});
};

export const createEvent = (client: Client, settings: EventSettings) => {
	mutation(client, '', EventLeaderboardAction.Create, settings);
};

export const updateEvent = (client: Client, leaderboardId: string, settings: EventSettings) => {
	mutation(client, leaderboardId, EventLeaderboardAction.Update, settings);
};

export const deleteEvent = (client: Client, leaderboardId: string) => {
	mutation(client, leaderboardId, EventLeaderboardAction.Delete, {
		name: '',
		startTime: '',
		endTime: ''
	});
};

export const togglePinEvent = (client: Client, leaderboardId: string) => {
	mutation(client, leaderboardId, EventLeaderboardAction.TogglePin, {
		name: '',
		startTime: '0',
		endTime: '0'
	});
};
