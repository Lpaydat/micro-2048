<!--
  ResilientLeaderboardLoader - Resilient leaderboard loading wrapper
  
  Provides:
  - Cache-first loading for instant display
  - Background revalidation
  - Error recovery with cached fallback
  - Loading states with skeleton
  
  Usage:
  <ResilientLeaderboardLoader leaderboardId={id}>
    {#snippet content({ leaderboard, balance, isStale, refresh })}
      <RankerLeaderboard ... />
    {/snippet}
  </ResilientLeaderboardLoader>
-->
<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { Snippet } from 'svelte';
	import { getClient } from '$lib/client';
	import { useLeaderboard, type LeaderboardState } from '$lib/hooks';
	import AsyncContent from '$lib/components/atoms/AsyncContent.svelte';
	import LoadingSkeleton from '$lib/components/atoms/LoadingSkeleton.svelte';

	interface Props {
		leaderboardId: string | undefined;
		/** Enable auto-polling */
		enablePolling?: boolean;
		/** Polling interval */
		pollingInterval?: number;
		/** Content to render when leaderboard is loaded */
		content: Snippet<[{
			leaderboard: LeaderboardState;
			balance: string | null;
			isStale: boolean;
			isLoading: boolean;
			refresh: () => Promise<void>;
		}]>;
		/** Optional loading snippet */
		loading?: Snippet;
		/** Optional error snippet */
		errorContent?: Snippet<[{ error: Error; retry: () => void }]>;
	}

	let {
		leaderboardId,
		enablePolling = true,
		pollingInterval = 5000,
		content,
		loading,
		errorContent
	}: Props = $props();

	// Create leaderboard hook
	const leaderboard = useLeaderboard(
		() => getClient(leaderboardId, true),
		leaderboardId,
		{
			enablePolling,
			pollingInterval,
			staleWhileRevalidate: true
		}
	);

	// Start polling when component mounts
	$effect(() => {
		if (enablePolling) {
			leaderboard.startPolling();
		}
		return () => {
			leaderboard.stopPolling();
		};
	});

	// Cleanup on destroy
	onDestroy(() => {
		leaderboard.destroy();
	});

	// Destructure reactive values
	const data = leaderboard.data;
	const balance = leaderboard.balance;
	const isLoading = leaderboard.isLoading;
	const isStale = leaderboard.isStale;
	const error = leaderboard.error;
	const isCircuitOpen = leaderboard.isCircuitOpen;
</script>

<AsyncContent
	isLoading={$isLoading && !$data}
	isStale={$isStale}
	error={$error}
	isCircuitOpen={$isCircuitOpen}
	onRetry={leaderboard.refresh}
	loadingOverlay={!!$data}
>
	{#snippet loading()}
		{#if loading}
			{@render loading()}
		{:else}
			<div class="flex flex-col gap-4 p-4">
				<LoadingSkeleton height="2.5rem" width="200px" rounded="lg" />
				<LoadingSkeleton height="3rem" rounded="md" />
				<div class="space-y-2">
					{#each Array(5) as _}
						<LoadingSkeleton height="4rem" rounded="lg" />
					{/each}
				</div>
			</div>
		{/if}
	{/snippet}

	{#snippet errorContent(ctx)}
		{#if errorContent}
			{@render errorContent(ctx)}
		{:else}
			<div class="flex flex-col items-center gap-4 p-8 text-center">
				<div class="text-5xl">😵</div>
				<div>
					<p class="text-lg font-medium text-red-300">Failed to load leaderboard</p>
					<p class="text-sm text-gray-400">{ctx.error.message}</p>
				</div>
				<button
					type="button"
					class="rounded-lg bg-red-800 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-red-700"
					onclick={ctx.retry}
				>
					Try Again
				</button>
			</div>
		{/if}
	{/snippet}

	{#if $data}
		{@render content({
			leaderboard: $data,
			balance: $balance,
			isStale: $isStale,
			isLoading: $isLoading,
			refresh: leaderboard.refresh
		})}
	{/if}
</AsyncContent>
