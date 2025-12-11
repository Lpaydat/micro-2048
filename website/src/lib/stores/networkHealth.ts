/**
 * Network Health Store
 * 
 * Tracks global network health status for the application.
 * Used to show indicators and adapt behavior during degraded conditions.
 */

import { writable, derived, type Readable } from 'svelte/store';
import { browser } from '$app/environment';

export type NetworkStatus = 'online' | 'degraded' | 'offline';

export interface NetworkHealthState {
	/** Current network status */
	status: NetworkStatus;
	/** Is browser online (basic connectivity) */
	isOnline: boolean;
	/** Recent request latency (ms, rolling average) */
	latency: number;
	/** Successful requests in last minute */
	successCount: number;
	/** Failed requests in last minute */
	failureCount: number;
	/** Last successful request timestamp */
	lastSuccessTime: number | null;
	/** Last failure timestamp */
	lastFailureTime: number | null;
	/** Error rate (0-1) */
	errorRate: number;
	/** Endpoints with open circuits */
	openCircuits: string[];
}

const initialState: NetworkHealthState = {
	status: 'online',
	isOnline: true,
	latency: 0,
	successCount: 0,
	failureCount: 0,
	lastSuccessTime: null,
	lastFailureTime: null,
	errorRate: 0,
	openCircuits: []
};

// Internal mutable state for metrics
const recentRequests: { timestamp: number; success: boolean; latency: number }[] = [];
const METRICS_WINDOW = 60000; // 1 minute window

function createNetworkHealthStore() {
	const { subscribe, set, update } = writable<NetworkHealthState>(initialState);

	// Cleanup old metrics periodically
	if (browser) {
		setInterval(() => {
			const cutoff = Date.now() - METRICS_WINDOW;
			while (recentRequests.length > 0 && recentRequests[0].timestamp < cutoff) {
				recentRequests.shift();
			}
			recalculateMetrics();
		}, 10000); // Every 10 seconds

		// Listen for online/offline events
		window.addEventListener('online', () => {
			update((state) => ({ ...state, isOnline: true }));
			recalculateMetrics();
		});

		window.addEventListener('offline', () => {
			update((state) => ({ ...state, isOnline: false, status: 'offline' }));
		});
	}

	function recalculateMetrics() {
		const cutoff = Date.now() - METRICS_WINDOW;
		const recent = recentRequests.filter((r) => r.timestamp >= cutoff);

		const successCount = recent.filter((r) => r.success).length;
		const failureCount = recent.filter((r) => !r.success).length;
		const totalCount = recent.length;

		const errorRate = totalCount > 0 ? failureCount / totalCount : 0;
		const avgLatency =
			recent.length > 0
				? recent.reduce((sum, r) => sum + r.latency, 0) / recent.length
				: 0;

		update((state) => {
			let status: NetworkStatus = 'online';

			if (!state.isOnline) {
				status = 'offline';
			} else if (errorRate > 0.5 || state.openCircuits.length > 0 || avgLatency > 10000) {
				status = 'degraded';
			}

			return {
				...state,
				status,
				latency: Math.round(avgLatency),
				successCount,
				failureCount,
				errorRate
			};
		});
	}

	return {
		subscribe,

		/**
		 * Record a successful request
		 */
		recordSuccess(latency: number = 0): void {
			const now = Date.now();
			recentRequests.push({ timestamp: now, success: true, latency });
			update((state) => ({
				...state,
				lastSuccessTime: now
			}));
			recalculateMetrics();
		},

		/**
		 * Record a failed request
		 */
		recordFailure(latency: number = 0): void {
			const now = Date.now();
			recentRequests.push({ timestamp: now, success: false, latency });
			update((state) => ({
				...state,
				lastFailureTime: now
			}));
			recalculateMetrics();
		},

		/**
		 * Update open circuits list
		 */
		setOpenCircuits(circuits: string[]): void {
			update((state) => ({
				...state,
				openCircuits: circuits
			}));
			recalculateMetrics();
		},

		/**
		 * Reset all metrics
		 */
		reset(): void {
			recentRequests.length = 0;
			set({
				...initialState,
				isOnline: browser ? navigator.onLine : true
			});
		},

		/**
		 * Get current state synchronously
		 */
		getState(): NetworkHealthState {
			let state = initialState;
			subscribe((s) => (state = s))();
			return state;
		}
	};
}

export const networkHealth = createNetworkHealthStore();

// Derived stores for easy access
export const isOnline: Readable<boolean> = derived(
	networkHealth,
	($health) => $health.isOnline
);

export const isDegraded: Readable<boolean> = derived(
	networkHealth,
	($health) => $health.status === 'degraded'
);

export const networkStatus: Readable<NetworkStatus> = derived(
	networkHealth,
	($health) => $health.status
);

export const hasOpenCircuits: Readable<boolean> = derived(
	networkHealth,
	($health) => $health.openCircuits.length > 0
);
