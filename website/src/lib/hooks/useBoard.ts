/**
 * useBoard - Smart hook for board data fetching
 * 
 * Features:
 * - Cache-first with background revalidation
 * - Automatic polling with adaptive intervals
 * - Circuit breaker integration
 * - Stale-while-revalidate pattern
 */

import { writable, type Readable } from 'svelte/store';
import { browser } from '$app/environment';
import type { Client } from '@urql/svelte';
import { gql } from '@urql/svelte';
import { cacheService, type BoardCacheData, TTL } from '$lib/services/cacheService';
import { requestManager, CircuitOpenError } from '$lib/services/requestManager';
import { pollingManager } from '$lib/services/pollingManager';

// Lightweight query for board state (excludes move history)
const GET_BOARD_STATE = gql`
	query BoardState($boardId: String!) {
		board(boardId: $boardId) {
			boardId
			board
			score
			isEnded
			player
			leaderboardId
			chainId
			shardId
			createdAt
			endTime
			totalMoves
		}
	}
`;

export interface UseBoardOptions {
	/** Enable automatic polling */
	enablePolling?: boolean;
	/** Polling interval in ms */
	pollingInterval?: number;
	/** Use stale-while-revalidate pattern */
	staleWhileRevalidate?: boolean;
	/** Skip initial fetch (useful if parent already has data) */
	skipInitialFetch?: boolean;
}

export interface BoardState {
	boardId: string;
	board: number[][];
	score: number;
	isEnded: boolean;
	player: string;
	leaderboardId: string;
	chainId: string;
	shardId?: string;
	createdAt: string;
	endTime?: string;
	totalMoves: number;
}

export interface UseBoardReturn {
	/** Current board data */
	data: Readable<BoardState | null>;
	/** Whether data is being fetched */
	isLoading: Readable<boolean>;
	/** Whether current data is stale */
	isStale: Readable<boolean>;
	/** Current error if any */
	error: Readable<Error | null>;
	/** Last successful update timestamp */
	lastUpdated: Readable<number | null>;
	/** Whether circuit breaker is open */
	isCircuitOpen: Readable<boolean>;

	/** Force refresh data */
	refresh: () => Promise<void>;
	/** Start polling */
	startPolling: () => void;
	/** Stop polling */
	stopPolling: () => void;
	/** Cleanup (call on component destroy) */
	destroy: () => void;
}

const DEFAULT_OPTIONS: UseBoardOptions = {
	enablePolling: true,
	pollingInterval: 2000,
	staleWhileRevalidate: true,
	skipInitialFetch: false
};

/**
 * Create a smart board data hook
 */
export function useBoard(
	getClient: () => Client,
	boardId: string | undefined,
	options: UseBoardOptions = {}
): UseBoardReturn {
	const opts = { ...DEFAULT_OPTIONS, ...options };
	const endpoint = `board:${boardId}`;
	const pollingId = `board-poll:${boardId}`;

	// Internal stores
	const dataStore = writable<BoardState | null>(null);
	const isLoadingStore = writable<boolean>(false);
	const isStaleStore = writable<boolean>(false);
	const errorStore = writable<Error | null>(null);
	const lastUpdatedStore = writable<number | null>(null);
	const isCircuitOpenStore = writable<boolean>(false);

	// Track if initialized
	let isInitialized = false;
	let isDestroyed = false;

	/**
	 * Fetch board data from network
	 */
	async function fetchFromNetwork(): Promise<BoardState | null> {
		if (!boardId || isDestroyed) return null;

		const client = getClient();

		try {
			const result = await requestManager.request(
				async () => {
					return new Promise<BoardState | null>((resolve, reject) => {
						const query = client.query(GET_BOARD_STATE, { boardId });

						query.subscribe((result) => {
							if (result.error) {
								reject(result.error);
							} else if (result.data !== undefined) {
								// Query completed when data is defined (even if null)
								resolve(result.data?.board || null);
							}
						});
					});
				},
				{
					endpoint,
					dedupKey: `board:${boardId}`,
					priority: 'high',
					timeout: 15000,
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
	 * Load data (cache-first, then network)
	 */
	async function loadData(forceNetwork: boolean = false): Promise<void> {
		if (!boardId || isDestroyed) return;

		isLoadingStore.set(true);
		errorStore.set(null);

		try {
			// Try cache first (unless force network)
			if (!forceNetwork && opts.staleWhileRevalidate) {
				const cached = await cacheService.getBoardStale(boardId);
				if (cached) {
					dataStore.set(cached.data as unknown as BoardState);
					isStaleStore.set(cached.isStale);
					lastUpdatedStore.set(Date.now() - (cached.isStale ? TTL.BOARD : 0));

					// If data is stale, continue to fetch from network
					if (!cached.isStale) {
						isLoadingStore.set(false);
						return;
					}
				}
			}

			// Fetch from network
			const networkData = await fetchFromNetwork();

			if (networkData && !isDestroyed) {
				dataStore.set(networkData);
				isStaleStore.set(false);
				lastUpdatedStore.set(Date.now());
				isCircuitOpenStore.set(false);

				// Update cache
				await cacheService.setBoard(boardId, networkData as unknown as BoardCacheData);
			}
		} catch (error) {
			console.error(`Error loading board ${boardId}:`, error);
			errorStore.set(error as Error);

			// Try to show cached data on error
			if (opts.staleWhileRevalidate) {
				const cached = await cacheService.getBoardStale(boardId);
				if (cached && !isDestroyed) {
					dataStore.set(cached.data as unknown as BoardState);
					isStaleStore.set(true);
				}
			}
		} finally {
			if (!isDestroyed) {
				isLoadingStore.set(false);
			}
		}
	}

	/**
	 * Initialize hook
	 */
	async function initialize(): Promise<void> {
		if (isInitialized || !boardId || !browser) return;
		isInitialized = true;

		// Try to load from cache immediately for instant UI
		const cached = await cacheService.getBoard(boardId);
		if (cached && !isDestroyed) {
			dataStore.set(cached as unknown as BoardState);
		}

		// Fetch from network
		if (!opts.skipInitialFetch) {
			await loadData();
		}

		// Setup polling
		if (opts.enablePolling) {
			pollingManager.register(pollingId, {
				fetcher: () => loadData(true),
				baseInterval: opts.pollingInterval!,
				minInterval: 1000,
				maxInterval: 30000,
				backoffFactor: 1.5,
				pauseWhenHidden: true,
				priority: 'high',
				immediate: false,
				onError: (error) => {
					console.warn(`Board polling error:`, error);
				}
			});
		}
	}

	// Initialize on creation (browser only)
	if (browser && boardId) {
		initialize();
	}

	return {
		data: { subscribe: dataStore.subscribe },
		isLoading: { subscribe: isLoadingStore.subscribe },
		isStale: { subscribe: isStaleStore.subscribe },
		error: { subscribe: errorStore.subscribe },
		lastUpdated: { subscribe: lastUpdatedStore.subscribe },
		isCircuitOpen: { subscribe: isCircuitOpenStore.subscribe },

		refresh: async () => {
			await loadData(true);
		},

		startPolling: () => {
			if (boardId && opts.enablePolling) {
				pollingManager.start(pollingId);
			}
		},

		stopPolling: () => {
			pollingManager.stop(pollingId);
		},

		destroy: () => {
			isDestroyed = true;
			pollingManager.unregister(pollingId);
		}
	};
}

export default useBoard;
