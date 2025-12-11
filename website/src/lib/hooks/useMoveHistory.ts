/**
 * useMoveHistory - Smart hook for paginated move history
 * 
 * Features:
 * - Chunked loading with prefetch
 * - Persistent cache (moves don't change)
 * - Smart prefetching based on user behavior
 * - Circuit breaker integration
 */

import { writable, type Readable } from 'svelte/store';
import { browser } from '$app/environment';
import type { Client } from '@urql/svelte';
import { gql } from '@urql/svelte';
import { cacheService } from '$lib/services/cacheService';
import { requestManager, CircuitOpenError } from '$lib/services/requestManager';

const GET_MOVES_PAGINATED = gql`
	query GetMovesPaginated($boardId: String!, $moveOffset: Int!, $moveLimit: Int!) {
		board(boardId: $boardId, moveOffset: $moveOffset, moveLimit: $moveLimit) {
			boardId
			totalMoves
			moveOffset
			moveLimit
			hasMoreMoves
			moveHistory {
				direction
				timestamp
				boardAfter
				scoreAfter
			}
		}
	}
`;

export interface MoveRecord {
	direction: string;
	timestamp: string;
	boardAfter: number[][];
	scoreAfter: number;
}

export interface LoadedRange {
	start: number;
	end: number;
}

export interface UseMoveHistoryOptions {
	/** Chunk size for loading */
	chunkSize?: number;
	/** Enable prefetching */
	enablePrefetch?: boolean;
	/** Number of moves to prefetch ahead */
	prefetchAhead?: number;
}

export interface UseMoveHistoryReturn {
	/** Total moves count */
	totalMoves: Readable<number>;
	/** Loaded ranges */
	loadedRanges: Readable<LoadedRange[]>;
	/** Whether currently loading */
	isLoading: Readable<boolean>;
	/** Current loading target */
	loadingTarget: Readable<number | null>;
	/** Current error if any */
	error: Readable<Error | null>;
	/** Whether circuit is open */
	isCircuitOpen: Readable<boolean>;

	/** Initialize with total moves count */
	initialize: (total: number) => void;
	/** Check if a move index is loaded */
	isMoveLoaded: (index: number) => boolean;
	/** Get move data at index (1-based) */
	getMove: (index: number) => MoveRecord | null;
	/** Load range around target move */
	loadRange: (targetMove: number) => Promise<void>;
	/** Prefetch around target */
	prefetch: (targetMove: number) => void;
	/** Clear all cached data */
	clear: () => Promise<void>;
	/** Cleanup */
	destroy: () => void;
}

const DEFAULT_OPTIONS: UseMoveHistoryOptions = {
	chunkSize: 100,
	enablePrefetch: true,
	prefetchAhead: 50
};

/**
 * Create a smart move history hook
 */
export function useMoveHistory(
	getClient: () => Client,
	boardId: string | undefined,
	options: UseMoveHistoryOptions = {}
): UseMoveHistoryReturn {
	const opts = { ...DEFAULT_OPTIONS, ...options };
	const endpoint = `moves:${boardId}`;

	// Internal state
	const moves: Map<number, MoveRecord> = new Map();
	const loadedChunks: Set<number> = new Set(); // Track which chunk start indices are loaded
	let totalMovesValue = 0;
	let isDestroyed = false;
	let pendingPrefetch: ReturnType<typeof setTimeout> | null = null;

	// Stores
	const totalMovesStore = writable<number>(0);
	const loadedRangesStore = writable<LoadedRange[]>([]);
	const isLoadingStore = writable<boolean>(false);
	const loadingTargetStore = writable<number | null>(null);
	const errorStore = writable<Error | null>(null);
	const isCircuitOpenStore = writable<boolean>(false);

	/**
	 * Calculate chunk start index for a move
	 */
	function getChunkStart(moveIndex: number): number {
		return Math.floor((moveIndex - 1) / opts.chunkSize!) * opts.chunkSize! + 1;
	}

	/**
	 * Update loaded ranges store
	 */
	function updateLoadedRanges(): void {
		const ranges: LoadedRange[] = [];
		const sortedChunks = Array.from(loadedChunks).sort((a, b) => a - b);

		let currentRange: LoadedRange | null = null;

		for (const chunkStart of sortedChunks) {
			const chunkEnd = Math.min(chunkStart + opts.chunkSize! - 1, totalMovesValue);

			if (currentRange && currentRange.end + 1 >= chunkStart) {
				// Extend current range
				currentRange.end = chunkEnd;
			} else {
				// Start new range
				if (currentRange) {
					ranges.push(currentRange);
				}
				currentRange = { start: chunkStart, end: chunkEnd };
			}
		}

		if (currentRange) {
			ranges.push(currentRange);
		}

		loadedRangesStore.set(ranges);
	}

	/**
	 * Fetch moves from network
	 */
	async function fetchFromNetwork(
		startIndex: number,
		limit: number
	): Promise<MoveRecord[]> {
		if (!boardId || isDestroyed) return [];

		const client = getClient();
		const offset = startIndex - 1; // Convert to 0-based offset

		try {
			const result = await requestManager.request(
				async () => {
					return new Promise<MoveRecord[]>((resolve, reject) => {
						const query = client.query(GET_MOVES_PAGINATED, {
							boardId,
							moveOffset: offset,
							moveLimit: limit
						});

						query.subscribe((result) => {
							if (result.error) {
								reject(result.error);
							} else if (result.data !== undefined) {
								resolve(result.data?.board?.moveHistory || []);
							}
						});
					});
				},
				{
					endpoint,
					dedupKey: `moves:${boardId}:${startIndex}:${limit}`,
					priority: 'high',
					timeout: 20000,
					retries: 2
				}
			);

			return result;
		} catch (error) {
			if (error instanceof CircuitOpenError) {
				isCircuitOpenStore.set(true);
			}
			throw error;
		}
	}

	/**
	 * Load a chunk of moves
	 */
	async function loadChunk(chunkStart: number): Promise<void> {
		if (!boardId || isDestroyed || loadedChunks.has(chunkStart)) return;

		// Try cache first
		const cached = await cacheService.getMoveChunk(boardId, chunkStart);
		if (cached) {
			// Add to memory
			for (let i = 0; i < cached.moves.length; i++) {
				moves.set(chunkStart + i, cached.moves[i]);
			}
			loadedChunks.add(chunkStart);
			updateLoadedRanges();
			return;
		}

		// Fetch from network
		const limit = Math.min(opts.chunkSize!, totalMovesValue - chunkStart + 1);
		const fetchedMoves = await fetchFromNetwork(chunkStart, limit);

		if (fetchedMoves.length > 0 && !isDestroyed) {
			// Add to memory
			for (let i = 0; i < fetchedMoves.length; i++) {
				moves.set(chunkStart + i, fetchedMoves[i]);
			}
			loadedChunks.add(chunkStart);
			updateLoadedRanges();

			// Cache for persistence
			await cacheService.setMoveChunk(boardId, chunkStart, fetchedMoves);
		}
	}

	/**
	 * Load range around target move
	 */
	async function loadRange(targetMove: number): Promise<void> {
		if (!boardId || isDestroyed || totalMovesValue === 0) return;

		// Clamp target to valid range
		targetMove = Math.max(1, Math.min(targetMove, totalMovesValue));

		const chunkStart = getChunkStart(targetMove);

		// Already loaded?
		if (loadedChunks.has(chunkStart)) return;

		isLoadingStore.set(true);
		loadingTargetStore.set(targetMove);
		errorStore.set(null);

		try {
			await loadChunk(chunkStart);
			isCircuitOpenStore.set(false);
		} catch (error) {
			console.error(`Error loading moves for ${boardId}:`, error);
			errorStore.set(error as Error);
		} finally {
			if (!isDestroyed) {
				isLoadingStore.set(false);
				loadingTargetStore.set(null);
			}
		}
	}

	/**
	 * Prefetch around target (debounced, low priority)
	 */
	function prefetch(targetMove: number): void {
		if (!opts.enablePrefetch || !boardId || isDestroyed) return;

		// Debounce prefetch
		if (pendingPrefetch) {
			clearTimeout(pendingPrefetch);
		}

		pendingPrefetch = setTimeout(async () => {
			if (isDestroyed) return;

			// Prefetch next chunk
			const nextChunkStart = getChunkStart(targetMove + opts.prefetchAhead!);
			if (nextChunkStart <= totalMovesValue && !loadedChunks.has(nextChunkStart)) {
				try {
					await loadChunk(nextChunkStart);
				} catch {
					// Silently fail prefetch
				}
			}

			// Prefetch previous chunk
			const prevChunkStart = getChunkStart(Math.max(1, targetMove - opts.prefetchAhead!));
			if (prevChunkStart >= 1 && !loadedChunks.has(prevChunkStart)) {
				try {
					await loadChunk(prevChunkStart);
				} catch {
					// Silently fail prefetch
				}
			}
		}, 500);
	}

	/**
	 * Initialize hook
	 */
	async function initializeFromCache(): Promise<void> {
		if (!browser || !boardId) return;

		// Load all cached chunks for this board
		const cachedChunks = await cacheService.getAllMoveChunks(boardId);
		for (const chunk of cachedChunks) {
			for (let i = 0; i < chunk.moves.length; i++) {
				moves.set(chunk.startIndex + i, chunk.moves[i]);
			}
			loadedChunks.add(chunk.startIndex);
		}

		if (loadedChunks.size > 0) {
			updateLoadedRanges();
		}
	}

	// Initialize cache on creation
	if (browser && boardId) {
		initializeFromCache();
	}

	return {
		totalMoves: { subscribe: totalMovesStore.subscribe },
		loadedRanges: { subscribe: loadedRangesStore.subscribe },
		isLoading: { subscribe: isLoadingStore.subscribe },
		loadingTarget: { subscribe: loadingTargetStore.subscribe },
		error: { subscribe: errorStore.subscribe },
		isCircuitOpen: { subscribe: isCircuitOpenStore.subscribe },

		initialize: (total: number) => {
			totalMovesValue = total;
			totalMovesStore.set(total);
		},

		isMoveLoaded: (index: number) => {
			return moves.has(index);
		},

		getMove: (index: number) => {
			return moves.get(index) || null;
		},

		loadRange,
		prefetch,

		clear: async () => {
			moves.clear();
			loadedChunks.clear();
			loadedRangesStore.set([]);
			if (boardId) {
				await cacheService.clearMoveChunks(boardId);
			}
		},

		destroy: () => {
			isDestroyed = true;
			if (pendingPrefetch) {
				clearTimeout(pendingPrefetch);
			}
		}
	};
}

export default useMoveHistory;
