import { getClient } from '$lib/client';
import { applicationId, port } from '$lib/constants';
import { newGame } from '$lib/graphql/mutations';
import { setBoardId } from '$lib/stores/boardId';
import { setGameCreationStatus } from '$lib/stores/gameStore';
import { userStore } from '$lib/stores/userStore';
import { hashSeed } from '$lib/utils/random';
import { get } from 'svelte/store';

/**
 * This function has side effects. It sets the boardId and gameCreationStatus.
 *
 * @param leaderboardId
 */
export const newGameBoard = async (leaderboardId: string = ''): Promise<string> => {
	const userInfo = get(userStore);
	const client = getClient(userInfo.chainId, applicationId, port);

	const seed = Math.floor(Math.random() * 10_000_000).toString();
	const timestamp = Date.now().toString();

	let boardId = '';
	boardId = (await hashSeed(seed, userInfo.username ?? '', timestamp)).toString();
	boardId = `${userInfo.chainId}.${boardId}`;
	setBoardId(boardId, leaderboardId);
	setGameCreationStatus(true);
	newGame(client, seed, timestamp, leaderboardId);

	return boardId;
};
