import type { GameKeys } from '$lib/game/models';
import { writable } from 'svelte/store';

// Enum for move directions
const directionList = {
	Up: 'Up',
	Down: 'Down',
	Left: 'Left',
	Right: 'Right'
};

const getDirection = (direction: GameKeys) => {
	const formattedDirection = direction?.replace('Arrow', '');
	if (formattedDirection && !Object.values(directionList).includes(formattedDirection)) {
		console.error('Invalid direction:', direction);
		return;
	}
	return formattedDirection;
};

export type MoveHistory = {
	direction: GameKeys;
	timestamp: string;
	boardId: string;
};

// Use Map for O(1) lookups and batch operations
const boardMoveMap = new Map<string, MoveHistory[]>();
export const moveHistoryStore = writable(boardMoveMap);

export const addMoveToHistory = (move: MoveHistory) => {
	moveHistoryStore.update((map) => {
		const moves = map.get(move.boardId) || [];
		moves.push(move);
		map.set(move.boardId, moves);
		return map;
	});
};

export const flushMoveHistory = (boardId: string) => {
	let moves: MoveHistory[] = [];
	moveHistoryStore.update((map) => {
		moves = map.get(boardId) || [];
		map.delete(boardId); // O(1) operation
		return map;
	});
	return moves;
};

export const getMoveBatchForSubmission = (moves: MoveHistory[]): string => {
	// Pre-allocate array for better performance
	const batch = new Array(moves.length);

	for (let i = 0; i < moves.length; i++) {
		batch[i] = [getDirection(moves[i].direction), moves[i].timestamp];
	}

	return JSON.stringify(batch);
};
