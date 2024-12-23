import { boardToString } from '$lib/game/utils';
import Queue from 'queue-fifo';
import { writable } from 'svelte/store';

// Create a writable store with an initial value of false
export const isNewGameCreated = writable(false);

export const boardSize = writable<'xs' | 'sm' | 'md' | 'lg'>('md');

// Create a queue to hold the board states
const boardQueue = new Queue<string>();

// Function to set the game creation status
export const setGameCreationStatus = (status: boolean) => {
	isNewGameCreated.set(status);
};

// Function to update the board state
export const updateBoardState = (newBoardState: string | undefined) => {
	if (newBoardState) {
		boardQueue.enqueue(newBoardState);
		// Optionally, limit the size of the queue
		if (boardQueue.size() > 10) {
			// keep only the last 10 states
			boardQueue.dequeue();
		}
	}
};

// Function to validate the board state against the backend state
export const validateBoardState = (
	backendState: string | number[][],
	isValid: boolean = false
): boolean => {
	if (!boardQueue.size()) {
		return isValid; // No states left in the queue, return current isValid
	}

	const board = typeof backendState === 'string' ? backendState : boardToString(backendState);
	const lastState = boardQueue.peek();

	if (lastState === board) {
		boardQueue.dequeue(); // Remove the matching state
		return true; // Found a match, return true
	} else {
		// boardQueue.dequeue(); // Remove the non-matching state
		// return validateBoardState(backendState, isValid); // Recurse with the next state
		return false;
	}
};
