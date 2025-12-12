import { getClient } from '$lib/client';
import { newGame } from '$lib/graphql/mutations';
import { userStore } from '$lib/stores/userStore';
import { get } from 'svelte/store';

/**
 * Create a new game board
 * @param leaderboardId - Tournament/leaderboard ID
 * @param timestamp - Current timestamp in milliseconds
 * @param rhythmTrackIndex - 🎵 Rhythm mode: which music track to use
 *   - undefined = no rhythm mode (or metronome mode)
 *   - 0+ = specific track index (already resolved from 'random' if needed)
 */
export const newGameBoard = async (
	leaderboardId: string,
	timestamp?: string,
	rhythmTrackIndex?: number
) => {
	// Get the player's chainId from userStore first
	const user = get(userStore);
	let playerChainId = user.chainId;

	// If not in store, try localStorage
	if (!playerChainId) {
		playerChainId = localStorage.getItem('chainId');
	}

	if (!playerChainId) {
		console.error('Player chainId not found. User must be logged in to create boards.');
		throw new Error('Player chainId not found. Please log in first.');
	}

	// Create board on the player's chain - smart contract will auto-select shard
	const client = getClient(playerChainId);
	const result = newGame(
		client,
		timestamp ?? Date.now().toString(),
		leaderboardId,
		rhythmTrackIndex
	);

	if (!result) {
		throw new Error('Failed to create board: authentication failed');
	}

	return result;
};
