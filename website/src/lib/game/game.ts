import { rndRange } from '$lib/utils/random';

import {
	boardToString,
	generateTabletFromMatrix,
	genNewTileValue,
	isEmptyTile,
	reverse,
	transpose
} from './utils';

import type { GameState, Row, Tablet, TileContent } from './models';
const genRow = (dimension: number): Row =>
	[...Array<TileContent>(dimension).keys()].map(() => ({
		value: 0,
		merged: false,
		new: false,
		swipe: false
	}));

const reposition = (tablet: Tablet): Tablet => {
	return tablet.map((row, rowIndex) =>
		row.map((tile, tileIndex) => ({
			...tile,
			new: false,
			...(tile.position ? { prevPosition: tile.position } : {}),
			...(tile.value > 0 ? { position: { top: rowIndex, left: tileIndex } } : {})
		}))
	);
};

const merge = (row: Row): Row => {
	const mergedRow: Row = [];
	let i = 0;

	while (i < row.length) {
		const current = row[i];
		const next = row[i + 1];

		if (current && next && current.value === next.value) {
			// Merge current and next
			mergedRow.push({ ...current, value: current.value + 1, merged: true });
			i += 2; // Skip the next tile as it has been merged
		} else {
			// Add current to mergedRow
			if (current) {
				mergedRow.push(current);
			}
			i += 1;
		}
	}

	return mergedRow;
};

const normalize = (tablet: Tablet): Tablet => {
	return tablet.map((row) => {
		const noZeroTiles = row.filter((tile) => !isEmptyTile(tile));
		const merged = merge(noZeroTiles);
		return [...merged, ...genRow(tablet.length - merged.length)];
	});
};

const getScore = (tablet: Tablet) =>
	tablet.reduce(
		(score: number, row: Row) =>
			row.reduce(
				(rowScore: number, tile: TileContent) => (tile.value > 0 ? 2 ** tile.value : 0) + rowScore,
				0
			) + score,
		0
	);

const nextState = (state: GameState, newTablet: Tablet): GameState => ({
	...state,
	finished: false,
	score: getScore(newTablet),
	tablet: newTablet
});

/**
 * Places a new tile on the board at the exact same position as Rust version would
 * @param board Current game board
 * @param tileValue Value to place (1 for 2, 2 for 4 - matching Rust's representation)
 * @param targetIndex Which empty cell to target (counting from bottom-right)
 * @returns New board with tile placed
 */
const spawnTile = async (
	board: Tablet,
	tileValue: number,
	targetIndex: number
): Promise<Tablet> => {
	let emptyCount = 0;
	// Traverse same way as Rust: bottom to top, right to left
	for (let row = 3; row >= 0; row--) {
		for (let col = 3; col >= 0; col--) {
			if (board[row][col].value === 0) {
				if (emptyCount === targetIndex) {
					board[row][col] = {
						value: tileValue,
						merged: false,
						new: true,
						swipe: false,
						position: { top: row, left: col }
					};
					return board;
				}
				emptyCount++;
			}
		}
	}

	return board;
};

const genNewTiles = async (
	tablet: Tablet,
	boardId: string,
	username: string,
	timestamp: string,
	prevTablet?: string
): Promise<Tablet> => {
	// Check if the board has changed
	const tabletString = boardToString(tablet);
	const hasChanged = tabletString !== prevTablet;
	if (!hasChanged) return tablet;

	const countEmptyTiles = tablet.flatMap((row) => row.filter((tile) => isEmptyTile(tile))).length;
	if (countEmptyTiles === 0) return tablet;

	const tileValue = await genNewTileValue(boardId, username, timestamp);
	const randomIndex = await rndRange(boardId, username, timestamp, 0, countEmptyTiles);
	return await spawnTile(tablet, tileValue, randomIndex);
};

export const genInitialState = (
	initialTablet: number[][],
	dimension: number,
	boardId: string,
	username: string
): GameState => {
	return {
		dimension,
		boardId,
		username,
		score: 0,
		finished: false,
		tablet: generateTabletFromMatrix(initialTablet),
		actions: {
			ArrowUp: async (state: GameState, timestamp: string, prevTablet?: string) =>
				nextState(
					state,
					await genNewTiles(
						reposition(transpose(normalize(transpose(state.tablet)))),
						boardId,
						username,
						timestamp,
						prevTablet
					)
				),
			ArrowDown: async (state: GameState, timestamp: string, prevTablet?: string) =>
				nextState(
					state,
					await genNewTiles(
						reposition(transpose(normalize(transpose(state.tablet.reverse()))).reverse()),
						boardId,
						username,
						timestamp,
						prevTablet
					)
				),
			ArrowLeft: async (state: GameState, timestamp: string, prevTablet?: string) =>
				nextState(
					state,
					await genNewTiles(
						reposition(normalize(state.tablet)),
						boardId,
						username,
						timestamp,
						prevTablet
					)
				),
			ArrowRight: async (state: GameState, timestamp: string, prevTablet?: string) =>
				nextState(
					state,
					await genNewTiles(
						reposition(reverse(normalize(reverse(state.tablet)))),
						boardId,
						username,
						timestamp,
						prevTablet
					)
				)
		}
	};
};
