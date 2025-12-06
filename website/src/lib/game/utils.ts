import { rndRange } from '../utils/random';

import type { Tablet, TileContent } from './models';

export const genNewTileValue = async (
	boardId: string,
	username: string,
	timestamp: string
): Promise<number> => {
	const random = await rndRange(boardId, username, timestamp, 0, 10);
	return random === 9 ? 2 : 1;
};

export const isEmptyTile = (tile: TileContent): boolean => tile.value === 0;

export const transpose = (matrix: Tablet): Tablet => {
	const n = matrix.length;
	const transposed: Tablet = Array.from({ length: n }, () => Array(n).fill(null));

	for (let i = 0; i < n; i++) {
		for (let j = 0; j < n; j++) {
			transposed[j][i] = matrix[i][j];
		}
	}

	return transposed;
};

export const reverse = (matrix: Tablet) => matrix.map((row) => row.reverse());

const indexes: Record<number, { top: number; left: number }> = {
	0: { top: 0, left: 0 },
	1: { top: 0, left: 1 },
	2: { top: 0, left: 2 },
	3: { top: 0, left: 3 },
	4: { top: 1, left: 0 },
	5: { top: 1, left: 1 },
	6: { top: 1, left: 2 },
	7: { top: 1, left: 3 },
	8: { top: 2, left: 0 },
	9: { top: 2, left: 1 },
	10: { top: 2, left: 2 },
	11: { top: 2, left: 3 },
	12: { top: 3, left: 0 },
	13: { top: 3, left: 1 },
	14: { top: 3, left: 2 },
	15: { top: 3, left: 3 }
};

export const generateTabletFromMatrix = (matrix: number[][]): Tablet => {
	return matrix?.map((row, rowIndex) =>
		row?.map((value, colIndex) => {
			const index = rowIndex * matrix.length + colIndex;
			return {
				value,
				merged: false,
				swipe: false,
				new: false,
				position: indexes[index],
				prevPosition: indexes[index]
			};
		})
	);
};

export const boardToString = (board?: Tablet | number[][]): string | undefined => {
	if (!board) return undefined;
	return board
		.flatMap((row) => row.map((tile) => (typeof tile === 'object' ? tile.value : tile)))
		.join(',');
};

/**
 * Spawns a tile on the board at a deterministic position based on seed.
 * Matches Rust's Game::spawn_tile logic - traverses from bottom-right to top-left.
 * @param board Current board as 2D array of values
 * @param tileValue Value to place (1 for 2, 2 for 4)
 * @param targetIndex Which empty cell to target (0-indexed from bottom-right)
 * @returns New board with tile placed
 */
const spawnTileOnMatrix = (board: number[][], tileValue: number, targetIndex: number): number[][] => {
	const newBoard = board.map(row => [...row]);
	let emptyCount = 0;
	
	// Traverse same way as Rust: bottom to top, right to left
	for (let row = 3; row >= 0; row--) {
		for (let col = 3; col >= 0; col--) {
			if (newBoard[row][col] === 0) {
				if (emptyCount === targetIndex) {
					newBoard[row][col] = tileValue;
					return newBoard;
				}
				emptyCount++;
			}
		}
	}
	
	return newBoard;
};

/**
 * Computes the initial board state for a game.
 * Matches Rust's Game::new() logic - spawns 2 tiles using timestamp and timestamp-1.
 * @param boardId The board ID
 * @param player The player username
 * @param createdAt The creation timestamp (in milliseconds from API)
 * @returns Promise resolving to 4x4 initial board matrix
 */
export const computeInitialBoard = async (
	boardId: string,
	player: string,
	createdAt: string
): Promise<number[][]> => {
	// Start with empty board
	let board: number[][] = [
		[0, 0, 0, 0],
		[0, 0, 0, 0],
		[0, 0, 0, 0],
		[0, 0, 0, 0]
	];
	
	// Convert milliseconds to match Rust (API returns milliseconds, Rust uses same)
	const timestamp = createdAt;
	const timestampMinus1 = (parseInt(timestamp) - 1).toString();
	
	// First tile spawn (uses timestamp)
	const emptyCount1 = 16; // All cells empty
	const tileValue1 = await genNewTileValue(boardId, player, timestamp);
	const targetIndex1 = await rndRange(boardId, player, timestamp, 0, emptyCount1);
	board = spawnTileOnMatrix(board, tileValue1, targetIndex1);
	
	// Second tile spawn (uses timestamp - 1)
	const emptyCount2 = 15; // One cell occupied
	const tileValue2 = await genNewTileValue(boardId, player, timestampMinus1);
	const targetIndex2 = await rndRange(boardId, player, timestampMinus1, 0, emptyCount2);
	board = spawnTileOnMatrix(board, tileValue2, targetIndex2);
	
	return board;
};
