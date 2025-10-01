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
	) {
		leaderboardAction(
			leaderboardId: $leaderboardId
			action: $action
			settings: $settings
			player: $player
			passwordHash: $passwordHash
		)
	}
`;

export type LeaderboardSettings = {
	name: string;
	description?: string;
	startTime: string;
	endTime: string;
	shardNumber?: number;
	baseTriggererCount?: number;
};

export type LeaderboardState = {
	leaderboardId: string;
	chainId: string;
	name: string;
	description?: string;
	isPinned: boolean;
	host: string;
	startTime: string;
	endTime: string;
	totalBoards: number;
	totalPlayers: number;
	shardIds: string[];
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
		return null;
	}

	return mutationStore({
		client,
		query: LEADERBOARD_ACTION,
		variables: { leaderboardId, action, player, passwordHash, settings }
	});
};

export const createLeaderboard = (client: Client, settings: LeaderboardSettings) => {
	return mutation(client, '', LeaderboardAction.Create, settings);
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
