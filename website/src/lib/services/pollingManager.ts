/**
 * Polling Manager - Adaptive polling with health-based throttling
 * 
 * Features:
 * - Adaptive intervals based on success/failure
 * - Pause when tab is hidden
 * - Global throttling during degraded conditions
 * - Priority-based polling (critical tasks poll faster)
 */

import { browser } from '$app/environment';
import { networkHealth } from '$lib/stores/networkHealth';

export type PollingPriority = 'critical' | 'high' | 'medium' | 'low';

export interface PollingConfig {
	/** Function to execute on each poll */
	fetcher: () => Promise<void>;
	/** Base interval in ms (normal conditions) */
	baseInterval: number;
	/** Minimum interval in ms (when active/critical) */
	minInterval: number;
	/** Maximum interval in ms (when errors occur) */
	maxInterval: number;
	/** Backoff multiplier on failure (e.g., 1.5) */
	backoffFactor: number;
	/** Whether to pause when tab is hidden */
	pauseWhenHidden: boolean;
	/** Priority level */
	priority: PollingPriority;
	/** Whether to run immediately on start */
	immediate: boolean;
	/** Optional callback on error */
	onError?: (error: Error) => void;
}

interface PollingTask {
	id: string;
	config: PollingConfig;
	currentInterval: number;
	consecutiveFailures: number;
	lastPollTime: number;
	timeoutId: ReturnType<typeof setTimeout> | null;
	isRunning: boolean;
	isPaused: boolean;
}

const DEFAULT_CONFIG: Omit<PollingConfig, 'fetcher'> = {
	baseInterval: 5000,
	minInterval: 1000,
	maxInterval: 60000,
	backoffFactor: 1.5,
	pauseWhenHidden: true,
	priority: 'medium',
	immediate: false
};

// Priority multipliers (lower = faster polling)
const PRIORITY_MULTIPLIERS: Record<PollingPriority, number> = {
	critical: 0.5,
	high: 0.75,
	medium: 1.0,
	low: 1.5
};

class PollingManager {
	private tasks: Map<string, PollingTask> = new Map();
	private globalThrottleMultiplier = 1.0;
	private isPageVisible = true;

	constructor() {
		if (browser) {
			// Listen for visibility changes
			document.addEventListener('visibilitychange', this.handleVisibilityChange.bind(this));

			// Subscribe to network health for adaptive throttling
			networkHealth.subscribe((health) => {
				if (health.status === 'degraded') {
					this.setGlobalThrottle(2.0); // Double all intervals
				} else if (health.status === 'offline') {
					this.setGlobalThrottle(5.0); // 5x intervals when offline
				} else {
					this.setGlobalThrottle(1.0); // Normal
				}
			});
		}
	}

	/**
	 * Register a new polling task
	 */
	register(id: string, config: Partial<PollingConfig> & { fetcher: () => Promise<void> }): void {
		const fullConfig: PollingConfig = { ...DEFAULT_CONFIG, ...config };

		const task: PollingTask = {
			id,
			config: fullConfig,
			currentInterval: fullConfig.baseInterval,
			consecutiveFailures: 0,
			lastPollTime: 0,
			timeoutId: null,
			isRunning: false,
			isPaused: false
		};

		this.tasks.set(id, task);
	}

	/**
	 * Start polling for a task
	 */
	start(id: string): void {
		const task = this.tasks.get(id);
		if (!task) {
			console.warn(`Polling task not found: ${id}`);
			return;
		}

		if (task.isRunning) return;

		task.isRunning = true;
		task.isPaused = false;

		if (task.config.immediate) {
			this.executePoll(task);
		} else {
			this.scheduleNextPoll(task);
		}
	}

	/**
	 * Stop polling for a task
	 */
	stop(id: string): void {
		const task = this.tasks.get(id);
		if (!task) return;

		if (task.timeoutId) {
			clearTimeout(task.timeoutId);
			task.timeoutId = null;
		}

		task.isRunning = false;
		task.isPaused = false;
	}

	/**
	 * Pause polling (e.g., when tab hidden)
	 */
	pause(id: string): void {
		const task = this.tasks.get(id);
		if (!task || !task.isRunning) return;

		if (task.timeoutId) {
			clearTimeout(task.timeoutId);
			task.timeoutId = null;
		}

		task.isPaused = true;
	}

	/**
	 * Resume polling
	 */
	resume(id: string): void {
		const task = this.tasks.get(id);
		if (!task || !task.isRunning || !task.isPaused) return;

		task.isPaused = false;
		this.scheduleNextPoll(task);
	}

	/**
	 * Stop all polling tasks
	 */
	stopAll(): void {
		for (const id of this.tasks.keys()) {
			this.stop(id);
		}
	}

	/**
	 * Pause all polling tasks
	 */
	pauseAll(): void {
		for (const id of this.tasks.keys()) {
			this.pause(id);
		}
	}

	/**
	 * Resume all polling tasks
	 */
	resumeAll(): void {
		for (const id of this.tasks.keys()) {
			this.resume(id);
		}
	}

	/**
	 * Unregister a task
	 */
	unregister(id: string): void {
		this.stop(id);
		this.tasks.delete(id);
	}

	/**
	 * Report success (reduces interval)
	 */
	reportSuccess(id: string): void {
		const task = this.tasks.get(id);
		if (!task) return;

		task.consecutiveFailures = 0;

		// Gradually reduce interval back to base
		const priorityMultiplier = PRIORITY_MULTIPLIERS[task.config.priority];
		const targetInterval = task.config.baseInterval * priorityMultiplier;

		if (task.currentInterval > targetInterval) {
			task.currentInterval = Math.max(
				targetInterval,
				task.currentInterval / task.config.backoffFactor
			);
		}
	}

	/**
	 * Report failure (increases interval)
	 */
	reportFailure(id: string): void {
		const task = this.tasks.get(id);
		if (!task) return;

		task.consecutiveFailures++;

		// Increase interval exponentially up to max
		task.currentInterval = Math.min(
			task.config.maxInterval,
			task.currentInterval * task.config.backoffFactor
		);
	}

	/**
	 * Trigger immediate poll (resets timer)
	 */
	pollNow(id: string): void {
		const task = this.tasks.get(id);
		if (!task || !task.isRunning) return;

		if (task.timeoutId) {
			clearTimeout(task.timeoutId);
			task.timeoutId = null;
		}

		this.executePoll(task);
	}

	/**
	 * Set global throttle multiplier
	 */
	setGlobalThrottle(multiplier: number): void {
		this.globalThrottleMultiplier = Math.max(0.5, Math.min(10, multiplier));
	}

	/**
	 * Get status of all tasks
	 */
	getStatus(): Map<
		string,
		{
			isRunning: boolean;
			isPaused: boolean;
			currentInterval: number;
			consecutiveFailures: number;
			lastPollTime: number;
		}
	> {
		const status = new Map();
		for (const [id, task] of this.tasks) {
			status.set(id, {
				isRunning: task.isRunning,
				isPaused: task.isPaused,
				currentInterval: task.currentInterval,
				consecutiveFailures: task.consecutiveFailures,
				lastPollTime: task.lastPollTime
			});
		}
		return status;
	}

	/**
	 * Get effective interval for a task
	 */
	getEffectiveInterval(id: string): number {
		const task = this.tasks.get(id);
		if (!task) return 0;
		return this.calculateEffectiveInterval(task);
	}

	private calculateEffectiveInterval(task: PollingTask): number {
		const priorityMultiplier = PRIORITY_MULTIPLIERS[task.config.priority];
		const effectiveInterval = task.currentInterval * priorityMultiplier * this.globalThrottleMultiplier;

		return Math.max(
			task.config.minInterval,
			Math.min(task.config.maxInterval, Math.round(effectiveInterval))
		);
	}

	private async executePoll(task: PollingTask): Promise<void> {
		if (!task.isRunning || task.isPaused) return;

		// Skip if page is hidden and task should pause
		if (!this.isPageVisible && task.config.pauseWhenHidden) {
			task.isPaused = true;
			return;
		}

		task.lastPollTime = Date.now();

		try {
			await task.config.fetcher();
			this.reportSuccess(task.id);
			networkHealth.recordSuccess(Date.now() - task.lastPollTime);
		} catch (error) {
			this.reportFailure(task.id);
			networkHealth.recordFailure(Date.now() - task.lastPollTime);
			task.config.onError?.(error as Error);
			console.warn(`Polling error for ${task.id}:`, error);
		}

		// Schedule next poll
		if (task.isRunning && !task.isPaused) {
			this.scheduleNextPoll(task);
		}
	}

	private scheduleNextPoll(task: PollingTask): void {
		if (task.timeoutId) {
			clearTimeout(task.timeoutId);
		}

		const interval = this.calculateEffectiveInterval(task);
		task.timeoutId = setTimeout(() => {
			this.executePoll(task);
		}, interval);
	}

	private handleVisibilityChange(): void {
		this.isPageVisible = document.visibilityState === 'visible';

		if (this.isPageVisible) {
			// Resume tasks that should run when visible
			for (const [id, task] of this.tasks) {
				if (task.isRunning && task.isPaused && task.config.pauseWhenHidden) {
					this.resume(id);
				}
			}
		} else {
			// Pause tasks that should pause when hidden
			for (const [id, task] of this.tasks) {
				if (task.isRunning && !task.isPaused && task.config.pauseWhenHidden) {
					this.pause(id);
				}
			}
		}
	}
}

// Singleton instance
export const pollingManager = new PollingManager();

export default PollingManager;
