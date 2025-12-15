/**
 * Circuit Breaker Pattern Implementation
 * 
 * Prevents cascading failures by stopping requests to failing endpoints.
 * 
 * States:
 * - CLOSED: Normal operation, requests go through
 * - OPEN: Failures exceeded threshold, requests are blocked
 * - HALF_OPEN: Testing if service recovered, limited requests allowed
 */

export type CircuitState = 'closed' | 'open' | 'half-open';

export interface CircuitBreakerConfig {
	/** Number of failures before opening circuit */
	failureThreshold: number;
	/** Time in ms before attempting recovery (half-open) */
	resetTimeout: number;
	/** Number of successful requests needed to close circuit */
	successThreshold: number;
	/** Optional callback when state changes */
	onStateChange?: (state: CircuitState, endpoint: string) => void;
}

interface CircuitStats {
	failures: number;
	successes: number;
	lastFailureTime: number;
	state: CircuitState;
}

const DEFAULT_CONFIG: CircuitBreakerConfig = {
	failureThreshold: 5,
	resetTimeout: 10000, // 10 seconds (reduced from 30s for better UX)
	successThreshold: 1 // Only need 1 success to recover (reduced from 2)
};

class CircuitBreaker {
	private circuits: Map<string, CircuitStats> = new Map();
	private config: CircuitBreakerConfig;

	constructor(config: Partial<CircuitBreakerConfig> = {}) {
		this.config = { ...DEFAULT_CONFIG, ...config };
	}

	/**
	 * Get or create circuit stats for an endpoint
	 */
	private getCircuit(endpoint: string): CircuitStats {
		if (!this.circuits.has(endpoint)) {
			this.circuits.set(endpoint, {
				failures: 0,
				successes: 0,
				lastFailureTime: 0,
				state: 'closed'
			});
		}
		return this.circuits.get(endpoint)!;
	}

	/**
	 * Check current state, potentially transitioning from open to half-open
	 */
	getState(endpoint: string): CircuitState {
		const circuit = this.getCircuit(endpoint);

		// Check if we should transition from open to half-open
		if (circuit.state === 'open') {
			const timeSinceFailure = Date.now() - circuit.lastFailureTime;
			if (timeSinceFailure >= this.config.resetTimeout) {
				this.setState(endpoint, 'half-open');
			}
		}

		return circuit.state;
	}

	/**
	 * Check if a request should be allowed
	 */
	canRequest(endpoint: string): boolean {
		const state = this.getState(endpoint);
		return state !== 'open';
	}

	/**
	 * Report a successful request
	 */
	recordSuccess(endpoint: string): void {
		const circuit = this.getCircuit(endpoint);

		if (circuit.state === 'half-open') {
			circuit.successes++;
			if (circuit.successes >= this.config.successThreshold) {
				this.setState(endpoint, 'closed');
				circuit.failures = 0;
				circuit.successes = 0;
			}
		} else if (circuit.state === 'closed') {
			// Reset failure count on success
			circuit.failures = Math.max(0, circuit.failures - 1);
		}
	}

	/**
	 * Report a failed request
	 */
	recordFailure(endpoint: string): void {
		const circuit = this.getCircuit(endpoint);
		circuit.failures++;
		circuit.lastFailureTime = Date.now();

		if (circuit.state === 'half-open') {
			// Any failure in half-open immediately opens circuit
			this.setState(endpoint, 'open');
			circuit.successes = 0;
		} else if (circuit.state === 'closed') {
			if (circuit.failures >= this.config.failureThreshold) {
				this.setState(endpoint, 'open');
			}
		}
	}

	/**
	 * Manually reset a circuit to closed state
	 */
	reset(endpoint: string): void {
		const circuit = this.getCircuit(endpoint);
		circuit.failures = 0;
		circuit.successes = 0;
		circuit.lastFailureTime = 0;
		this.setState(endpoint, 'closed');
	}

	/**
	 * Reset all circuits
	 */
	resetAll(): void {
		this.circuits.clear();
	}

	/**
	 * Get stats for monitoring
	 */
	getStats(endpoint: string): CircuitStats & { config: CircuitBreakerConfig } {
		return {
			...this.getCircuit(endpoint),
			config: this.config
		};
	}

	/**
	 * Get all circuit states for monitoring
	 */
	getAllStats(): Map<string, CircuitStats> {
		// Trigger state updates for all circuits
		for (const endpoint of this.circuits.keys()) {
			this.getState(endpoint);
		}
		return new Map(this.circuits);
	}

	private setState(endpoint: string, state: CircuitState): void {
		const circuit = this.getCircuit(endpoint);
		const previousState = circuit.state;
		circuit.state = state;

		if (previousState !== state) {
			this.config.onStateChange?.(state, endpoint);
		}
	}
}

// Singleton instance for the application
export const circuitBreaker = new CircuitBreaker({
	failureThreshold: 5,
	resetTimeout: 10000, // 10 seconds - recover faster
	successThreshold: 1 // Only 1 success needed
});

export default CircuitBreaker;
