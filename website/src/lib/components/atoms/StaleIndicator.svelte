<!--
  StaleIndicator - Shows when data is stale
  
  Displays a subtle indicator that data may be outdated,
  with optional refresh button.
-->
<script lang="ts">
	import RefreshCw from 'lucide-svelte/icons/refresh-cw';
	import { fade } from 'svelte/transition';

	interface Props {
		/** Whether data is stale */
		isStale: boolean;
		/** Last updated timestamp */
		lastUpdated?: number | null;
		/** Whether currently refreshing */
		isRefreshing?: boolean;
		/** Callback to trigger refresh */
		onRefresh?: () => void;
		/** Size variant */
		size?: 'sm' | 'md';
		/** Show as inline or floating */
		variant?: 'inline' | 'floating';
	}

	let {
		isStale,
		lastUpdated = null,
		isRefreshing = false,
		onRefresh,
		size = 'sm',
		variant = 'inline'
	}: Props = $props();

	// Format time ago
	function formatTimeAgo(timestamp: number): string {
		const seconds = Math.floor((Date.now() - timestamp) / 1000);

		if (seconds < 5) return 'just now';
		if (seconds < 60) return `${seconds}s ago`;
		if (seconds < 3600) return `${Math.floor(seconds / 60)}m ago`;
		if (seconds < 86400) return `${Math.floor(seconds / 3600)}h ago`;
		return `${Math.floor(seconds / 86400)}d ago`;
	}

	const timeAgo = $derived(lastUpdated ? formatTimeAgo(lastUpdated) : null);

	const sizeClasses = $derived({
		sm: 'text-xs gap-1 px-2 py-1',
		md: 'text-sm gap-2 px-3 py-1.5'
	}[size]);

	const iconSize = $derived(size === 'sm' ? 12 : 14);

	const variantClasses = $derived({
		inline: '',
		floating: 'absolute top-2 right-2 shadow-lg'
	}[variant]);
</script>

{#if isStale}
	<div
		class="flex items-center rounded-full bg-yellow-900/50 text-yellow-300 {sizeClasses} {variantClasses}"
		transition:fade={{ duration: 150 }}
	>
		{#if timeAgo}
			<span class="opacity-75">Updated {timeAgo}</span>
		{:else}
			<span class="opacity-75">Data may be stale</span>
		{/if}

		{#if onRefresh}
			<button
				type="button"
				class="ml-1 rounded-full p-1 transition-colors hover:bg-yellow-800/50"
				onclick={onRefresh}
				disabled={isRefreshing}
				title="Refresh"
			>
				<RefreshCw
					size={iconSize}
					class={isRefreshing ? 'animate-spin' : ''}
				/>
			</button>
		{/if}
	</div>
{/if}
