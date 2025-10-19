import { writable } from 'svelte/store';
import type { Writable } from 'svelte/store';

// Define proper types for move data
export interface MoveHistoryRecord {
	direction: string; // "Up", "Down", "Left", "Right"
	timestamp: string; // milliseconds
	boardAfter: number[][]; // 4x4 board
	scoreAfter: number;
}

// Types for move history pagination
export interface MoveRange {
	start: number; // 1-based move index
	end: number; // inclusive
	moves: MoveHistoryRecord[]; // Move data
}

export interface MoveHistoryCache {
	boardId: string;
	totalMoves: number;
	loadedRanges: MoveRange[];
	isLoading: boolean;
	loadingTarget?: number;
}

// Define the store interface
export interface PaginatedMoveHistoryStore {
	subscribe: Writable<MoveHistoryCache>['subscribe'];
	initialize(totalMoves: number): void;
	isMoveLoaded(moveIndex: number): boolean;
	getMove(moveIndex: number): MoveHistoryRecord | null;
	addLoadedRange(start: number, moves: MoveHistoryRecord[]): void;
	setLoading(isLoading: boolean, target?: number): void;
	getLoadedRanges(): MoveRange[];
	reset(): void;
}

// Store for each board's paginated move history
const paginatedHistoryStores = new Map<string, PaginatedMoveHistoryStore>();

// Get or create store for a specific board
export function getPaginatedMoveHistory(boardId: string): PaginatedMoveHistoryStore {
	if (!paginatedHistoryStores.has(boardId)) {
		const initialState: MoveHistoryCache = {
			boardId,
			totalMoves: 0,
			loadedRanges: [],
			isLoading: false
		};

		const store = writable(initialState);

		// Helper functions
		const { subscribe, set, update } = store;

		return {
			subscribe,

			// Initialize with total moves count
			initialize(totalMoves: number) {
				update((state) => ({ ...state, totalMoves }));
			},

			// Check if a move is already loaded
			isMoveLoaded(moveIndex: number): boolean {
				let loaded = false;
				subscribe((state) => {
					loaded = state.loadedRanges.some(
						(range) => moveIndex >= range.start && moveIndex <= range.end
					);
				})();
				return loaded;
			},

			// Get move data if loaded
			getMove(moveIndex: number): MoveHistoryRecord | null {
				let move = null;
				subscribe((state) => {
					const range = state.loadedRanges.find(
						(range) => moveIndex >= range.start && moveIndex <= range.end
					);
					if (range) {
						move = range.moves[moveIndex - range.start];
					}
				})();
				return move;
			},

			// Add new loaded range
			addLoadedRange(start: number, moves: MoveHistoryRecord[]) {
				update((state) => {
					const end = start + moves.length - 1;
					const newRange: MoveRange = { start, end, moves };

					// Merge overlapping or adjacent ranges
					const mergedRanges = [...state.loadedRanges, newRange]
						.sort((a, b) => a.start - b.start)
						.reduce((acc: MoveRange[], range) => {
							const last = acc[acc.length - 1];
							if (last && range.start <= last.end + 1) {
								// Merge overlapping/adjacent ranges
								last.end = Math.max(last.end, range.end);
								last.moves = [...last.moves, ...range.moves];
							} else {
								acc.push(range);
							}
							return acc;
						}, []);

					return {
						...state,
						loadedRanges: mergedRanges,
						isLoading: false,
						loadingTarget: undefined
					};
				});
			},

			// Set loading state
			setLoading(isLoading: boolean, target?: number) {
				update((state) => ({
					...state,
					isLoading,
					loadingTarget: target
				}));
			},

			// Get loaded ranges for UI display
			getLoadedRanges(): MoveRange[] {
				let ranges: MoveRange[] = [];
				subscribe((state) => {
					ranges = state.loadedRanges;
				})();
				return ranges;
			},

			// Reset cache for new board
			reset() {
				set({
					boardId,
					totalMoves: 0,
					loadedRanges: [],
					isLoading: false
				});
			}
		};
	}

	return paginatedHistoryStores.get(boardId)!;
}

// Helper function to calculate what range to load based on target
export function calculateLoadRange(
	targetMove: number,
	totalMoves: number,
	batchSize: number = 200
) {
	// Ensure target is within bounds
	targetMove = Math.max(1, Math.min(targetMove, totalMoves));

	// Calculate range centered on target, but adjusted for boundaries
	let start = Math.max(1, targetMove - Math.floor(batchSize / 2));
	const end = Math.min(totalMoves, start + batchSize - 1);

	// Adjust start if we hit the upper boundary
	if (end - start + 1 < batchSize) {
		start = Math.max(1, end - batchSize + 1);
	}

	return { start, end, limit: end - start + 1 };
}
