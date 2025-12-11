/**
 * useLeaderboard - Smart hook for leaderboard data fetching
 * 
 * Features:
 * - Cache-first with aggressive revalidation
 * - Automatic polling with adaptive intervals
 * - Circuit breaker integration
 * - Stale-while-revalidate pattern
 */

import { writable, type Readable } from 'svelte/store';
import { browser } from '$app/environment';
import type { Client } from '@urql/svelte';
import { gql } from '@urql/svelte';
import { cacheService, type LeaderboardCacheData, TTL } from '$lib/services/cacheService';
import { requestManager, CircuitOpenError } from '$lib/services/requestManager';
import { pollingManager } from '$lib/services/pollingManager';

const GET_LEADERBOARD = gql`
	query Leaderboard {
		leaderboard {
			leaderboardId
			name
			description
			host
			startTime
			endTime
			totalBoards
			totalPlayers
			isPinned
			lastUpdate
			rankers {
				username
				score
				boardId
				isEnded
			}
			shardIds
		}
		balance
	}
`;

export interface UseLeaderboardOptions {
	/** Enable automatic polling */
	enablePolling?: boolean;
	/** Polling interval in ms */
	pollingInterval?: number;
	/** Use stale-while-revalidate pattern */
	staleWhileRevalidate?: boolean;
	/** Skip initial fetch */
	skipInitialFetch?: boolean;
}

export interface Ranker {
	username: string;
	score: number;
	boardId: string;
	isEnded?: boolean;
}

export interface LeaderboardState {
	leaderboardId: string;
	name: string;
	description?: string;
	host: string;
	startTime: string;
	endTime: string;
	totalBoards: number;
	totalPlayers: number;
	isPinned?: boolean;
	lastUpdate?: string;
	rankers: Ranker[];
	shardIds?: string[];
	balance?: string;
}

export interface UseLeaderboardReturn {
	/** Current leaderboard data */
	data: Readable<LeaderboardState | null>;
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
	/** Balance from query */
	balance: Readable<string | null>;

	/** Force refresh data */
	refresh: () => Promise<void>;
	/** Start polling */
	startPolling: () => void;
	/** Stop polling */
	stopPolling: () => void;
	/** Cleanup (call on component destroy) */
	destroy: () => void;
}

const DEFAULT_OPTIONS: UseLeaderboardOptions = {
	enablePolling: true,
	pollingInterval: 5000,
	staleWhileRevalidate: true,
	skipInitialFetch: false
};

/**
 * Create a smart leaderboard data hook
 */
export function useLeaderboard(
	getClient: () => Client,
	leaderboardId: string | undefined,
	options: UseLeaderboardOptions = {}
): UseLeaderboardReturn {
	const opts = { ...DEFAULT_OPTIONS, ...options };
	const cacheId = leaderboardId || 'main';
	const endpoint = `leaderboard:${cacheId}`;
	const pollingId = `leaderboard-poll:${cacheId}`;

	// Internal stores
	const dataStore = writable<LeaderboardState | null>(null);
	const isLoadingStore = writable<boolean>(false);
	const isStaleStore = writable<boolean>(false);
	const errorStore = writable<Error | null>(null);
	const lastUpdatedStore = writable<number | null>(null);
	const isCircuitOpenStore = writable<boolean>(false);
	const balanceStore = writable<string | null>(null);

	// Track if initialized
	let isInitialized = false;
	let isDestroyed = false;

	/**
	 * Fetch leaderboard data from network
	 */
	async function fetchFromNetwork(): Promise<{ leaderboard: LeaderboardState | null; balance: string | null }> {
		if (isDestroyed) return { leaderboard: null, balance: null };

		const client = getClient();

		try {
			const result = await requestManager.request(
				async () => {
					return new Promise<{ leaderboard: LeaderboardState | null; balance: string | null }>((resolve, reject) => {
						const query = client.query(GET_LEADERBOARD, {});

						query.subscribe((result) => {
							if (result.error) {
								reject(result.error);
							} else if (result.data !== undefined) {
								resolve({
									leaderboard: result.data?.leaderboard || null,
									balance: result.data?.balance || null
								});
							}
						});
					});
				},
				{
					endpoint,
					dedupKey: `leaderboard:${cacheId}`,
					priority: 'medium',
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
		if (isDestroyed) return;

		isLoadingStore.set(true);
		errorStore.set(null);

		try {
			// Try cache first (unless force network)
			if (!forceNetwork && opts.staleWhileRevalidate) {
				const cached = await cacheService.getLeaderboardStale(cacheId);
				if (cached) {
					dataStore.set(cached.data as unknown as LeaderboardState);
					isStaleStore.set(cached.isStale);
					lastUpdatedStore.set(Date.now() - (cached.isStale ? TTL.LEADERBOARD : 0));

					// If data is fresh, don't fetch from network yet
					if (!cached.isStale) {
						isLoadingStore.set(false);
						return;
					}
				}
			}

			// Fetch from network
			const { leaderboard, balance } = await fetchFromNetwork();

			if (!isDestroyed) {
				if (leaderboard) {
					dataStore.set(leaderboard);
					isStaleStore.set(false);
					lastUpdatedStore.set(Date.now());
					isCircuitOpenStore.set(false);

					// Update cache
					await cacheService.setLeaderboard(cacheId, leaderboard as unknown as LeaderboardCacheData);
				}

				if (balance !== null) {
					balanceStore.set(balance);
				}
			}
		} catch (error) {
			console.error(`Error loading leaderboard ${cacheId}:`, error);
			errorStore.set(error as Error);

			// Try to show cached data on error
			if (opts.staleWhileRevalidate) {
				const cached = await cacheService.getLeaderboardStale(cacheId);
				if (cached && !isDestroyed) {
					dataStore.set(cached.data as unknown as LeaderboardState);
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
		if (isInitialized || !browser) return;
		isInitialized = true;

		// Try to load from cache immediately for instant UI
		const cached = await cacheService.getLeaderboard(cacheId);
		if (cached && !isDestroyed) {
			dataStore.set(cached as unknown as LeaderboardState);
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
				minInterval: 2000,
				maxInterval: 60000,
				backoffFactor: 1.5,
				pauseWhenHidden: true,
				priority: 'medium',
				immediate: false,
				onError: (error) => {
					console.warn(`Leaderboard polling error:`, error);
				}
			});
		}
	}

	// Initialize on creation (browser only)
	if (browser) {
		initialize();
	}

	return {
		data: { subscribe: dataStore.subscribe },
		isLoading: { subscribe: isLoadingStore.subscribe },
		isStale: { subscribe: isStaleStore.subscribe },
		error: { subscribe: errorStore.subscribe },
		lastUpdated: { subscribe: lastUpdatedStore.subscribe },
		isCircuitOpen: { subscribe: isCircuitOpenStore.subscribe },
		balance: { subscribe: balanceStore.subscribe },

		refresh: async () => {
			await loadData(true);
		},

		startPolling: () => {
			if (opts.enablePolling) {
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

export default useLeaderboard;
