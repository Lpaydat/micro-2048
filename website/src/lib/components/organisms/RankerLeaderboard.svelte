<script lang="ts">
	import RankerCard from '../molecules/RankerCard.svelte';

	interface Props {
		leaderboardId?: string;
		name?: string;
		host?: string;
		startDate?: string;
		endDate?: string;
		rankers?: {
			username: string;
			score: number;
			boardId: string;
		}[];
	}

	let { leaderboardId, name, host, startDate, endDate, rankers = [] }: Props = $props();
</script>

<div class="mx-auto mt-4 md:mt-8 flex h-[calc(100vh-8rem)] w-full max-w-4xl flex-col overflow-hidden">
	<h1 class="mb-4 md:mb-6 text-center text-2xl md:text-4xl font-extrabold text-gray-100">Leaderboard</h1>
	<div class="flex-1 overflow-hidden bg-gray-800 px-2 py-6 shadow-xl lg:rounded-lg lg:p-6">
		{#if (rankers?.length ?? 0) > 0}
			<div class="flex h-full flex-col overflow-visible">
				<!-- Fixed Header Row -->
				<div
					class="flex items-center justify-between border-b border-gray-700 px-4 pb-2 text-xs font-semibold text-gray-400 lg:text-base"
				>
					<span class="w-1/12 text-left">#</span>
					<span class="w-5/12">Player</span>
					<span class="w-3/12">Board ID</span>
					<span class="w-3/12 text-right">Score</span>
				</div>
				<!-- Scrollable Player Rows -->
				<div class="flex-1 space-y-4 overflow-y-auto overflow-x-hidden pt-4">
					{#each rankers as player, index}
						<RankerCard {player} rank={index + 1} />
					{/each}
				</div>
			</div>
		{:else}
			<div class="flex flex-col text-md md:text-lg h-full items-center justify-center">
				<p class="text-center font-semibold text-gray-400">No players yet.</p>
				<p class="text-center font-semibold text-gray-400">Be the first to join and claim the top spot!</p>
			</div>
		{/if}
	</div>
</div>
