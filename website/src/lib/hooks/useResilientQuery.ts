/**
 * useResilientQuery - Wraps urql queries with resilience features
 * 
 * Provides a drop-in enhancement for existing queryStore usage:
 * - Timeout protection
 * - Circuit breaker integration
 * - Network health tracking
 * - Retry with backoff
 */

import { writable, type Readable } from 'svelte/store';
import { browser } from '$app/environment';
import type { Client, AnyVariables, DocumentInput, OperationResult } from '@urql/svelte';
import { requestManager, CircuitOpenError } from '$lib/services/requestManager';
import { networkHealth } from '$lib/stores/networkHealth';

export interface ResilientQueryOptions {
	/** Endpoint identifier for circuit breaker */
	endpoint?: string;
	/** Request timeout in ms */
	timeout?: number;
	/** Number of retries */
	retries?: number;
	/** Priority level */
	priority?: 'critical' | 'high' | 'medium' | 'low';
}

export interface ResilientQueryResult<T> {
	/** Current data */
	data: Readable<T | null>;
	/** Whether query is loading */
	isLoading: Readable<boolean>;
	/** Current error */
	error: Readable<Error | null>;
	/** Whether circuit is open */
	isCircuitOpen: Readable<boolean>;
	/** Re-execute the query */
	reexecute: (options?: { requestPolicy?: 'network-only' | 'cache-first' }) => Promise<void>;
}

const DEFAULT_OPTIONS: ResilientQueryOptions = {
	endpoint: 'graphql',
	timeout: 15000,
	retries: 2,
	priority: 'medium'
};

/**
 * Execute a query with resilience features
 */
export function useResilientQuery<T, V extends AnyVariables = AnyVariables>(
	client: Client,
	query: DocumentInput<T, V>,
	variables: V,
	options: ResilientQueryOptions = {}
): ResilientQueryResult<T> {
	const opts = { ...DEFAULT_OPTIONS, ...options };

	// Internal stores
	const dataStore = writable<T | null>(null);
	const isLoadingStore = writable<boolean>(false);
	const errorStore = writable<Error | null>(null);
	const isCircuitOpenStore = writable<boolean>(false);

	/**
	 * Execute the query with resilience
	 */
	async function execute(requestPolicy: 'network-only' | 'cache-first' = 'network-only'): Promise<void> {
		if (!browser) return;

		isLoadingStore.set(true);
		errorStore.set(null);

		const startTime = Date.now();

		try {
			const result = await requestManager.request<OperationResult<T, V>>(
				async () => {
					return new Promise<OperationResult<T, V>>((resolve, reject) => {
						const queryResult = client.query<T, V>(query, variables, { requestPolicy });

						queryResult.subscribe((result) => {
							if (result.error) {
								reject(result.error);
							} else if (result.data !== undefined) {
								resolve(result);
							}
						});
					});
				},
				{
					endpoint: opts.endpoint,
					priority: opts.priority,
					timeout: opts.timeout,
					retries: opts.retries
				}
			);

			if (result.data) {
				dataStore.set(result.data);
				isCircuitOpenStore.set(false);
				networkHealth.recordSuccess(Date.now() - startTime);
			}
		} catch (error) {
			console.error('Resilient query error:', error);
			errorStore.set(error as Error);
			networkHealth.recordFailure(Date.now() - startTime);

			if (error instanceof CircuitOpenError) {
				isCircuitOpenStore.set(true);
			}
		} finally {
			isLoadingStore.set(false);
		}
	}

	// Execute initial query
	if (browser) {
		execute();
	}

	return {
		data: { subscribe: dataStore.subscribe },
		isLoading: { subscribe: isLoadingStore.subscribe },
		error: { subscribe: errorStore.subscribe },
		isCircuitOpen: { subscribe: isCircuitOpenStore.subscribe },
		reexecute: async (options) => {
			await execute(options?.requestPolicy || 'network-only');
		}
	};
}

/**
 * Create a resilient query store factory
 * Use this to create queries that share circuit breaker state
 */
export function createResilientQueryFactory(defaultOptions: ResilientQueryOptions = {}) {
	return function <T, V extends AnyVariables = AnyVariables>(
		client: Client,
		query: DocumentInput<T, V>,
		variables: V,
		options: ResilientQueryOptions = {}
	): ResilientQueryResult<T> {
		return useResilientQuery(client, query, variables, { ...defaultOptions, ...options });
	};
}

export default useResilientQuery;
