import { Client, gql, mutationStore } from '@urql/svelte';

const CREATE_EVENT = gql`
	mutation CreateEvent(
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
};

export const createEvent = (client: Client, settings: EventSettings) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return;
	}

	const timestamp = Date.now().toString();

	console.log('player', player);
	console.log('passwordHash', passwordHash);
	console.log('settings', settings);
	console.log('timestamp', timestamp);

	delete settings.description;

	mutationStore({
		client,
		query: CREATE_EVENT,
		variables: { leaderboardId: '', action: 'Create', player, passwordHash, settings, timestamp }
	});
};
