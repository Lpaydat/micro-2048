import { getClient } from '$lib/client';
import { newGame } from '$lib/graphql/mutations';
import { userStore } from '$lib/stores/userStore';
import { get } from 'svelte/store';

export const newGameBoard = async (
	leaderboardId: string,
	timestamp?: string
) => {
	// Get the player's chainId from userStore first
	let user = get(userStore);
	let playerChainId = user.chainId;
	
	// If not in store, try localStorage
	if (!playerChainId) {
		playerChainId = localStorage.getItem('chainId');
	}
	
	if (!playerChainId) {
		console.error('Player chainId not found. User must be logged in to create boards.');
		throw new Error('Player chainId not found. Please log in first.');
	}
	
	console.log('Creating board on player chain:', playerChainId);
	console.log('For leaderboard:', leaderboardId);
	console.log('Smart contract will auto-select shard');
	
	// Create board on the player's chain - smart contract will auto-select shard
	const client = getClient(playerChainId);
	const result = newGame(client, timestamp ?? Date.now().toString(), leaderboardId);
	
	if (!result) {
		throw new Error('Failed to create board: authentication failed');
	}
	
	return result;
};
