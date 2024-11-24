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
	return matrix.map((row, rowIndex) =>
		row.map((value, colIndex) => {
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
