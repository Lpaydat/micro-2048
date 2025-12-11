<!--
  ResilientBoardLoader - Resilient board loading wrapper
  
  Provides:
  - Cache-first loading for instant display
  - Background revalidation
  - Error recovery with cached fallback
  - Loading states with skeleton
  
  Usage:
  <ResilientBoardLoader boardId={id} chainId={chainId}>
    {#snippet content({ board, isStale, refresh })}
      <Game ... />
    {/snippet}
  </ResilientBoardLoader>
-->
<script lang="ts">
	import { onDestroy } from 'svelte';
	import type { Snippet } from 'svelte';
	import { getClient } from '$lib/client';
	import { useBoard, type BoardState } from '$lib/hooks';
	import AsyncContent from '$lib/components/atoms/AsyncContent.svelte';
	import LoadingSkeleton from '$lib/components/atoms/LoadingSkeleton.svelte';

	interface Props {
		boardId: string | undefined;
		chainId: string | undefined;
		/** Enable auto-polling */
		enablePolling?: boolean;
		/** Content to render when board is loaded */
		content: Snippet<[{
			board: BoardState;
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
		boardId,
		chainId,
		enablePolling = true,
		content,
		loading,
		errorContent
	}: Props = $props();

	// Create board hook
	const board = useBoard(
		() => getClient(chainId),
		boardId,
		{
			enablePolling,
			pollingInterval: 2000,
			staleWhileRevalidate: true
		}
	);

	// Start polling when component mounts
	$effect(() => {
		if (enablePolling && boardId) {
			board.startPolling();
		}
		return () => {
			board.stopPolling();
		};
	});

	// Cleanup on destroy
	onDestroy(() => {
		board.destroy();
	});

	// Destructure reactive values
	const data = board.data;
	const isLoading = board.isLoading;
	const isStale = board.isStale;
	const error = board.error;
	const isCircuitOpen = board.isCircuitOpen;
</script>

<AsyncContent
	isLoading={$isLoading && !$data}
	isStale={$isStale}
	error={$error}
	isCircuitOpen={$isCircuitOpen}
	onRetry={board.refresh}
	loadingOverlay={!!$data}
>
	{#snippet loading()}
		{#if loading}
			{@render loading()}
		{:else}
			<div class="flex flex-col gap-4 p-4">
				<LoadingSkeleton height="3rem" rounded="lg" />
				<LoadingSkeleton height="20rem" rounded="lg" />
				<LoadingSkeleton height="2rem" rounded="md" lines={2} />
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
					<p class="text-lg font-medium text-red-300">Failed to load game</p>
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
			board: $data,
			isStale: $isStale,
			isLoading: $isLoading,
			refresh: board.refresh
		})}
	{/if}
</AsyncContent>
