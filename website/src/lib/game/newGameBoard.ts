import { getClient } from '$lib/client';
import { newGame } from '$lib/graphql/mutations';

export const newGameBoard = async (
	leaderboardId: string,
	shardId?: string,
	timestamp?: string
): Promise<void> => {
	const client = getClient(shardId ?? leaderboardId);
	newGame(client, timestamp ?? Date.now().toString());
};
