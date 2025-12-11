<!--
  AsyncContent - Wrapper for async content with loading/error states
  
  Provides:
  - Loading skeleton
  - Error state with retry
  - Stale data indicator
  - Graceful degradation
-->
<script lang="ts">
	import { fade } from 'svelte/transition';
	import StaleIndicator from './StaleIndicator.svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		/** Whether data is loading */
		isLoading: boolean;
		/** Whether data is stale */
		isStale?: boolean;
		/** Current error */
		error?: Error | null;
		/** Last updated timestamp */
		lastUpdated?: number | null;
		/** Whether circuit breaker is open */
		isCircuitOpen?: boolean;
		/** Callback to retry/refresh */
		onRetry?: () => void;
		/** Show loading overlay instead of replacing content */
		loadingOverlay?: boolean;
		/** Custom loading content */
		loading?: Snippet;
		/** Custom error content */
		errorContent?: Snippet<[{ error: Error; retry: () => void }]>;
		/** Main content */
		children?: Snippet;
	}

	let {
		isLoading,
		isStale = false,
		error = null,
		lastUpdated = null,
		isCircuitOpen = false,
		onRetry,
		loadingOverlay = false,
		loading,
		errorContent,
		children
	}: Props = $props();

	// Show content if we have children, even when loading (for stale-while-revalidate)
	const showContent = $derived(!error || loadingOverlay);

	function handleRetry() {
		if (onRetry) {
			onRetry();
		}
	}
</script>

<div class="async-content relative">
	<!-- Stale indicator -->
	{#if isStale && !isLoading}
		<div class="absolute right-2 top-2 z-10">
			<StaleIndicator
				{isStale}
				{lastUpdated}
				isRefreshing={isLoading}
				onRefresh={onRetry}
			/>
		</div>
	{/if}

	<!-- Circuit open warning -->
	{#if isCircuitOpen}
		<div
			class="mb-2 rounded-lg bg-yellow-900/50 px-3 py-2 text-sm text-yellow-300"
			transition:fade={{ duration: 150 }}
		>
			<span class="font-medium">Connection issues</span>
			<span class="opacity-75"> - Some features may be limited</span>
			{#if onRetry}
				<button
					type="button"
					class="ml-2 underline hover:no-underline"
					onclick={handleRetry}
				>
					Retry
				</button>
			{/if}
		</div>
	{/if}

	<!-- Error state -->
	{#if error && !loadingOverlay}
		<div
			class="flex flex-col items-center justify-center gap-4 rounded-lg bg-red-900/20 p-6 text-center"
			transition:fade={{ duration: 200 }}
		>
			{#if errorContent}
				{@render errorContent({ error, retry: handleRetry })}
			{:else}
				<div class="text-4xl">😵</div>
				<div>
					<p class="font-medium text-red-300">Something went wrong</p>
					<p class="text-sm text-red-400/75">{error.message}</p>
				</div>
				{#if onRetry}
					<button
						type="button"
						class="rounded-lg bg-red-800 px-4 py-2 text-sm font-medium text-red-100 transition-colors hover:bg-red-700"
						onclick={handleRetry}
					>
						Try Again
					</button>
				{/if}
			{/if}
		</div>
	{/if}

	<!-- Content -->
	{#if showContent}
		<div
			class="transition-opacity duration-200"
			class:opacity-50={isLoading && loadingOverlay}
			class:pointer-events-none={isLoading && loadingOverlay}
		>
			{#if children}
				{@render children()}
			{/if}
		</div>
	{/if}

	<!-- Loading state -->
	{#if isLoading && !loadingOverlay}
		<div
			class="flex items-center justify-center p-6"
			transition:fade={{ duration: 200 }}
		>
			{#if loading}
				{@render loading()}
			{:else}
				<div class="flex items-center gap-3">
					<div
						class="h-5 w-5 animate-spin rounded-full border-2 border-gray-400 border-t-transparent"
					></div>
					<span class="text-gray-400">Loading...</span>
				</div>
			{/if}
		</div>
	{/if}

	<!-- Loading overlay -->
	{#if isLoading && loadingOverlay}
		<div
			class="pointer-events-none absolute inset-0 flex items-center justify-center bg-gray-900/30"
			transition:fade={{ duration: 150 }}
		>
			<div
				class="h-8 w-8 animate-spin rounded-full border-2 border-gray-400 border-t-transparent"
			></div>
		</div>
	{/if}
</div>

<style>
	.async-content {
		min-height: 2rem;
	}
</style>
