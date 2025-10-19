<script lang="ts">
	import { isMobile } from '$lib/stores/isMobile';
	import { formatBalance } from '$lib/utils/formatBalance';
	import LeaderboardDetails from '../molecules/LeaderboardDetails.svelte';
	import RankerCard from '../molecules/RankerCard.svelte';

	interface Props {
		hasSubHeader?: boolean;
		leaderboardId?: string; // This is actually the chainId from URL
		leaderboardChainId?: string; // Explicit chain ID prop
		name?: string;
		description?: string;
		host?: string;
		startTime?: string;
		endTime?: string;
		totalBoards?: number;
		totalPlayers?: number;
		balance?: number;
		rankers?: {
			username: string;
			score: number;
			boardId: string;
			isEnded?: boolean;
		}[];
	}

	let {
		rankers = [],
		leaderboardId,
		leaderboardChainId,
		hasSubHeader = false,
		balance,
		description,
		...rest
	}: Props = $props();

	// Use leaderboardChainId if provided, otherwise fall back to leaderboardId from props
	const chainId = $derived(leaderboardChainId || leaderboardId);

	// Check if description should be shown
	const hasDescription = $derived(chainId && description && description.trim());

	const height = $derived(
		$isMobile
			? hasSubHeader
				? 'h-[calc(100vh-152px)] md:h-[calc(100vh-170px)] lg:h-[calc(100vh-180px)]'
				: 'h-[calc(100vh-95px)] md:h-[calc(100vh-100px)]'
			: hasSubHeader
				? 'h-[calc(100vh-152px)] md:h-[calc(100vh-120px)]'
				: 'h-[calc(100vh-95px)] md:h-[calc(100vh-68px)]'
	);

	const sortedRankers = $derived(rankers.slice().sort((a: any, b: any) => b.score - a.score));

	const isTournamentEnded = $derived.by(() => {
		const end = Number(rest.endTime ?? '0');
		if (end === 0) return false;
		return end <= Date.now();
	});

	let isDescriptionExpanded = $state(false);
</script>

<div
	class="mx-auto mt-4 flex {height} w-full flex-col overflow-hidden {hasDescription
		? 'max-w-7xl xl:flex-row xl:gap-4'
		: 'max-w-4xl'}"
>
	<!-- Main Leaderboard Section -->
	<div class="flex flex-1 flex-col overflow-hidden {hasDescription ? 'xl:max-w-4xl' : ''}">
		<div class="flex flex-col gap-3 md:flex-row md:gap-6">
			<div class="flex flex-row items-center gap-4">
				<h1
					class="md:ms-none mb-2 text-center text-2xl font-extrabold text-gray-100 md:mb-3 md:text-4xl"
				>
					Leaderboard
				</h1>
				<span
					class="mb-2 rounded-lg bg-gray-800 px-3 py-1 text-sm font-medium text-gray-300 shadow-sm"
				>
					<span class="opacity-75">Balance:</span>
					<span class="ml-1 font-mono">{formatBalance(balance)}</span>
				</span>
			</div>
			{#if leaderboardId}
				<LeaderboardDetails {...rest} />
			{/if}
		</div>

		<!-- Mobile: Collapsible Description -->
		{#if hasDescription}
			<div class="mb-3 mt-4 xl:hidden">
				<button
					onclick={() => (isDescriptionExpanded = !isDescriptionExpanded)}
					class="w-full rounded bg-gray-700/60 px-3 py-1.5 text-left text-xs font-medium text-gray-300 transition-colors hover:bg-gray-700/80"
				>
					<div class="flex items-center justify-between">
						<span>Event Description</span>
						<span class="text-xs transition-transform {isDescriptionExpanded ? 'rotate-180' : ''}"
							>▼</span
						>
					</div>
				</button>
				{#if isDescriptionExpanded}
					<div class="mt-2 rounded bg-black/40 p-3 text-xs leading-relaxed text-gray-300">
						{description}
					</div>
				{/if}
			</div>
		{/if}

		<div class="flex-1 overflow-hidden bg-black/40 px-2 py-6 shadow-xl lg:rounded-lg lg:p-6">
			{#if (sortedRankers?.length ?? 0) > 0}
				<div class="flex h-full flex-col overflow-visible">
					<div
						class="flex items-center justify-between border-b border-gray-700 px-4 pb-2 text-xs font-semibold text-gray-400 lg:text-base"
					>
						<span class="w-1/12 text-left">#</span>
						<span class="w-5/12">Player</span>
						<span class="w-3/12">Board ID</span>
						<span class="w-3/12 text-right">Score</span>
					</div>
					<div class="flex-1 snap-y space-y-4 overflow-y-auto overflow-x-hidden pt-4">
						{#each sortedRankers as player, index}
							<RankerCard {player} rank={index + 1} {isTournamentEnded} />
						{/each}
					</div>
				</div>
			{:else}
				<div class="text-md flex h-full flex-col items-center justify-center md:text-lg">
					<p class="text-center font-semibold text-gray-400">No players yet.</p>
					<p class="text-center font-semibold text-gray-400">
						Be the first to join and claim the top spot!
					</p>
				</div>
			{/if}
		</div>
	</div>

	<!-- Desktop: Right Sidebar for Description (only on xl+ screens) -->
	{#if hasDescription}
		<aside class="hidden xl:block xl:w-96">
			<div
				class="sticky top-4 flex h-[calc(100vh-120px)] flex-col overflow-hidden rounded-lg bg-black/40 shadow-xl"
			>
				<div class="flex-none border-b border-gray-700 px-6 py-4">
					<h2 class="text-xl font-bold text-[#EEE4DA]">Event Description</h2>
				</div>
				<div class="flex-1 overflow-y-auto px-6 py-4">
					<p class="whitespace-pre-wrap text-sm leading-relaxed text-gray-300">
						{description}
					</p>
				</div>
			</div>
		</aside>
	{/if}
</div>
