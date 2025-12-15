/**
 * Request Manager - Central request handling with:
 * - Request deduplication (same query returns existing promise)
 * - Priority queue (critical > high > medium > low)
 * - Timeout handling
 * - Retry with exponential backoff
 * - Circuit breaker integration
 */

import { circuitBreaker, type CircuitState } from './circuitBreaker';

export type RequestPriority = 'critical' | 'high' | 'medium' | 'low';

export interface RequestOptions {
	/** Request priority for queue ordering */
	priority?: RequestPriority;
	/** Timeout in milliseconds */
	timeout?: number;
	/** Number of retry attempts */
	retries?: number;
	/** Base delay for exponential backoff (ms) */
	retryDelay?: number;
	/** Unique key for deduplication */
	dedupKey?: string;
	/** Endpoint identifier for circuit breaker */
	endpoint?: string;
	/** Skip circuit breaker check */
	bypassCircuitBreaker?: boolean;
}

interface PendingRequest<T> {
	promise: Promise<T>;
	timestamp: number;
}

interface QueuedRequest {
	id: string;
	priority: RequestPriority;
	execute: () => Promise<void>;
	timestamp: number;
}

const PRIORITY_ORDER: Record<RequestPriority, number> = {
	critical: 0,
	high: 1,
	medium: 2,
	low: 3
};

const DEFAULT_OPTIONS: Required<Omit<RequestOptions, 'dedupKey' | 'endpoint'>> = {
	priority: 'medium',
	timeout: 15000, // 15 seconds
	retries: 2,
	retryDelay: 1000,
	bypassCircuitBreaker: false
};

class RequestManager {
	private pendingRequests: Map<string, PendingRequest<unknown>> = new Map();
	private requestQueue: QueuedRequest[] = [];
	private isProcessingQueue = false;
	private maxConcurrent = 6; // Browser limit per domain
	private activeRequests = 0;
	private requestIdCounter = 0;

	/**
	 * Execute a request with full management (dedup, timeout, retry, circuit breaker)
	 */
	async request<T>(
		fetcher: () => Promise<T>,
		options: RequestOptions = {}
	): Promise<T> {
		const opts = { ...DEFAULT_OPTIONS, ...options };
		const endpoint = opts.endpoint || 'default';
		const dedupKey = opts.dedupKey;

		// Check circuit breaker
		if (!opts.bypassCircuitBreaker && !circuitBreaker.canRequest(endpoint)) {
			const state = circuitBreaker.getState(endpoint);
			throw new CircuitOpenError(endpoint, state);
		}

		// Check for existing in-flight request with same key
		if (dedupKey) {
			const existing = this.pendingRequests.get(dedupKey);
			if (existing) {
				return existing.promise as Promise<T>;
			}
		}

		// Create the request promise with retry logic
		const requestPromise = this.executeWithRetry(fetcher, opts, endpoint);

		// Track for deduplication
		if (dedupKey) {
			this.pendingRequests.set(dedupKey, {
				promise: requestPromise,
				timestamp: Date.now()
			});

			// Clean up after completion
			requestPromise.finally(() => {
				this.pendingRequests.delete(dedupKey);
			});
		}

		return requestPromise;
	}

	/**
	 * Queue a request for priority-based execution
	 */
	queueRequest<T>(
		fetcher: () => Promise<T>,
		options: RequestOptions = {}
	): Promise<T> {
		return new Promise((resolve, reject) => {
			const id = `req-${++this.requestIdCounter}`;
			const priority = options.priority || 'medium';

			const queuedRequest: QueuedRequest = {
				id,
				priority,
				timestamp: Date.now(),
				execute: async () => {
					try {
						const result = await this.request(fetcher, options);
						resolve(result);
					} catch (error) {
						reject(error);
					}
				}
			};

			// Insert in priority order
			const insertIndex = this.requestQueue.findIndex(
				(req) => PRIORITY_ORDER[req.priority] > PRIORITY_ORDER[priority]
			);

			if (insertIndex === -1) {
				this.requestQueue.push(queuedRequest);
			} else {
				this.requestQueue.splice(insertIndex, 0, queuedRequest);
			}

			this.processQueue();
		});
	}

	/**
	 * Cancel all pending requests (useful on navigation)
	 */
	cancelAll(): void {
		this.requestQueue = [];
		this.pendingRequests.clear();
	}

	/**
	 * Cancel requests by priority
	 */
	cancelByPriority(priority: RequestPriority): void {
		this.requestQueue = this.requestQueue.filter((req) => req.priority !== priority);
	}

	/**
	 * Get current queue status
	 */
	getStatus(): {
		queueLength: number;
		activeRequests: number;
		pendingDedup: number;
	} {
		return {
			queueLength: this.requestQueue.length,
			activeRequests: this.activeRequests,
			pendingDedup: this.pendingRequests.size
		};
	}

	/**
	 * Get circuit breaker state for an endpoint
	 */
	getCircuitState(endpoint: string): CircuitState {
		return circuitBreaker.getState(endpoint);
	}

	/**
	 * Reset circuit breaker for an endpoint
	 */
	resetCircuit(endpoint: string): void {
		circuitBreaker.reset(endpoint);
	}

	private async executeWithRetry<T>(
		fetcher: () => Promise<T>,
		opts: Required<Omit<RequestOptions, 'dedupKey' | 'endpoint'>> & { endpoint?: string },
		endpoint: string
	): Promise<T> {
		let lastError: Error | null = null;

		for (let attempt = 0; attempt <= opts.retries; attempt++) {
			try {
				const result = await this.executeWithTimeout(fetcher, opts.timeout);
				circuitBreaker.recordSuccess(endpoint);
				return result;
			} catch (error) {
				lastError = error as Error;
				circuitBreaker.recordFailure(endpoint);

				// Don't retry on circuit open
				if (error instanceof CircuitOpenError) {
					throw error;
				}

				// Don't retry on timeout if it's the last attempt
				if (attempt < opts.retries) {
					const delay = opts.retryDelay * Math.pow(2, attempt);
					await this.sleep(delay);

					// Re-check circuit breaker before retry
					if (!opts.bypassCircuitBreaker && !circuitBreaker.canRequest(endpoint)) {
						throw new CircuitOpenError(endpoint, circuitBreaker.getState(endpoint));
					}
				}
			}
		}

		throw lastError || new Error('Request failed');
	}

	private executeWithTimeout<T>(
		fetcher: () => Promise<T>,
		timeout: number
	): Promise<T> {
		return new Promise((resolve, reject) => {
			const timeoutId = setTimeout(() => {
				reject(new TimeoutError(timeout));
			}, timeout);

			fetcher()
				.then((result) => {
					clearTimeout(timeoutId);
					resolve(result);
				})
				.catch((error) => {
					clearTimeout(timeoutId);
					reject(error);
				});
		});
	}

	private async processQueue(): Promise<void> {
		if (this.isProcessingQueue) return;
		this.isProcessingQueue = true;

		while (this.requestQueue.length > 0 && this.activeRequests < this.maxConcurrent) {
			const request = this.requestQueue.shift();
			if (!request) break;

			this.activeRequests++;
			request.execute().finally(() => {
				this.activeRequests--;
				// Continue processing queue after request completes
				if (this.requestQueue.length > 0) {
					this.processQueue();
				}
			});
		}

		this.isProcessingQueue = false;
	}

	private sleep(ms: number): Promise<void> {
		return new Promise((resolve) => setTimeout(resolve, ms));
	}
}

// Custom error types
export class TimeoutError extends Error {
	constructor(public timeout: number) {
		super(`Request timed out after ${timeout}ms`);
		this.name = 'TimeoutError';
	}
}

export class CircuitOpenError extends Error {
	constructor(
		public endpoint: string,
		public state: CircuitState
	) {
		super(`Circuit breaker is ${state} for endpoint: ${endpoint}`);
		this.name = 'CircuitOpenError';
	}
}

// Singleton instance
export const requestManager = new RequestManager();

export default RequestManager;
