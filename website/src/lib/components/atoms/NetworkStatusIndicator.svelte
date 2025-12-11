<!--
  NetworkStatusIndicator - Shows current network health
  
  Displays:
  - Online/Offline status
  - Degraded performance warning
  - Circuit breaker status
-->
<script lang="ts">
	import { networkHealth, networkStatus, hasOpenCircuits } from '$lib/stores/networkHealth';
	import { fade, slide } from 'svelte/transition';

	// Props
	interface Props {
		/** Show expanded details */
		showDetails?: boolean;
		/** Position */
		position?: 'top-right' | 'bottom-right' | 'top-left' | 'bottom-left';
		/** Only show when degraded/offline */
		onlyShowProblems?: boolean;
	}

	let {
		showDetails = false,
		position = 'bottom-right',
		onlyShowProblems = true
	}: Props = $props();

	// Computed
	const shouldShow = $derived(
		!onlyShowProblems || $networkStatus !== 'online' || $hasOpenCircuits
	);

	const statusColor = $derived({
		online: 'bg-green-500',
		degraded: 'bg-yellow-500',
		offline: 'bg-red-500'
	}[$networkStatus]);

	const statusText = $derived({
		online: 'Connected',
		degraded: 'Slow Connection',
		offline: 'Offline'
	}[$networkStatus]);

	const positionClasses = $derived({
		'top-right': 'top-4 right-4',
		'bottom-right': 'bottom-4 right-4',
		'top-left': 'top-4 left-4',
		'bottom-left': 'bottom-4 left-4'
	}[position]);

	let isExpanded = $state(false);
</script>

{#if shouldShow}
	<div
		class="fixed {positionClasses} z-50"
		transition:fade={{ duration: 200 }}
	>
		<button
			type="button"
			class="flex items-center gap-2 rounded-lg bg-gray-900/90 px-3 py-2 text-sm shadow-lg backdrop-blur-sm transition-all hover:bg-gray-800/90"
			onclick={() => (isExpanded = !isExpanded)}
		>
			<!-- Status dot -->
			<span class="relative flex h-3 w-3">
				<span
					class="absolute inline-flex h-full w-full animate-ping rounded-full opacity-75 {statusColor}"
				></span>
				<span class="relative inline-flex h-3 w-3 rounded-full {statusColor}"></span>
			</span>

			<!-- Status text -->
			<span class="text-gray-200">{statusText}</span>

			{#if $hasOpenCircuits}
				<span class="text-xs text-yellow-400">(Limited)</span>
			{/if}
		</button>

		<!-- Expanded details -->
		{#if isExpanded || showDetails}
			<div
				class="mt-2 rounded-lg bg-gray-900/95 p-4 text-sm shadow-lg backdrop-blur-sm"
				transition:slide={{ duration: 200 }}
			>
				<div class="space-y-2">
					<div class="flex justify-between">
						<span class="text-gray-400">Latency</span>
						<span class="font-mono text-gray-200">{$networkHealth.latency}ms</span>
					</div>

					<div class="flex justify-between">
						<span class="text-gray-400">Success Rate</span>
						<span class="font-mono text-gray-200">
							{Math.round((1 - $networkHealth.errorRate) * 100)}%
						</span>
					</div>

					<div class="flex justify-between">
						<span class="text-gray-400">Requests (1m)</span>
						<span class="font-mono">
							<span class="text-green-400">{$networkHealth.successCount}</span>
							<span class="text-gray-500">/</span>
							<span class="text-red-400">{$networkHealth.failureCount}</span>
						</span>
					</div>

					{#if $networkHealth.openCircuits.length > 0}
						<div class="border-t border-gray-700 pt-2">
							<span class="text-yellow-400">Circuit breakers open:</span>
							<ul class="mt-1 text-xs text-gray-400">
								{#each $networkHealth.openCircuits as circuit}
									<li>{circuit}</li>
								{/each}
							</ul>
							<button
								type="button"
								class="mt-2 rounded bg-yellow-600 px-2 py-1 text-xs text-white hover:bg-yellow-500"
								onclick={async () => {
									const { circuitBreaker } = await import('$lib/services/circuitBreaker');
									circuitBreaker.resetAll();
									networkHealth.setOpenCircuits([]);
								}}
							>
								Reset All Circuits
							</button>
						</div>
					{/if}

					{#if $networkHealth.lastSuccessTime}
						<div class="border-t border-gray-700 pt-2 text-xs text-gray-500">
							Last success: {new Date($networkHealth.lastSuccessTime).toLocaleTimeString()}
						</div>
					{/if}
				</div>
			</div>
		{/if}
	</div>
{/if}
