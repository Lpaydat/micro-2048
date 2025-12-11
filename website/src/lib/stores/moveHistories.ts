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

/**
 * Flush only the first N moves from the history.
 * This is used after a successful sync to remove only the moves that were submitted,
 * preserving any moves that were added during the sync.
 */
export const flushNMoves = (boardId: string, count: number): MoveHistory[] => {
	let flushedMoves: MoveHistory[] = [];
	moveHistoryStore.update((map) => {
		const moves = map.get(boardId) || [];
		if (count >= moves.length) {
			// Flush all moves
			flushedMoves = moves;
			map.delete(boardId);
		} else {
			// Only flush first N moves, keep the rest
			flushedMoves = moves.slice(0, count);
			map.set(boardId, moves.slice(count));
		}
		return map;
	});
	return flushedMoves;
};

/**
 * Flush all moves with timestamp <= maxTimestamp.
 * This is used when backend confirms processing - we flush moves up to the
 * timestamp that was confirmed, keeping any newer moves that arrived after submission.
 * 
 * Returns the number of moves flushed.
 */
export const flushMovesUpToTimestamp = (boardId: string, maxTimestamp: number): number => {
	let flushedCount = 0;
	moveHistoryStore.update((map) => {
		const moves = map.get(boardId) || [];
		if (moves.length === 0) return map;
		
		// Find the split point - first move with timestamp > maxTimestamp
		const splitIndex = moves.findIndex(m => parseInt(m.timestamp) > maxTimestamp);
		
		if (splitIndex === -1) {
			// All moves have timestamp <= maxTimestamp, flush all
			flushedCount = moves.length;
			map.delete(boardId);
		} else if (splitIndex === 0) {
			// No moves to flush (all are newer than maxTimestamp)
			flushedCount = 0;
		} else {
			// Flush moves up to splitIndex, keep the rest
			flushedCount = splitIndex;
			map.set(boardId, moves.slice(splitIndex));
		}
		return map;
	});
	return flushedCount;
};

export const getMoveBatchForSubmission = (moves: MoveHistory[]): string => {
	const validMoves = moves.filter((m) =>
		Object.values(directionList).includes(m.direction.replace('Arrow', ''))
	);

	// Pre-allocate array using valid moves length
	const batch = new Array(validMoves.length);

	for (let i = 0; i < validMoves.length; i++) {
		const dir = getDirection(validMoves[i].direction);
		if (!dir) continue;
		batch[i] = [dir, validMoves[i].timestamp];
	}

	const result = JSON.stringify(batch.filter(Boolean));
	
	// 🔍 DEBUG: Log the batch being submitted
	console.log(`📦 Move batch for submission (${validMoves.length} moves):`, result);
	
	return result;
};
