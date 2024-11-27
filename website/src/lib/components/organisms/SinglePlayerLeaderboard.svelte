<script lang="ts">
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import RankerCard from '../molecules/RankerCard.svelte';
	import { onMount } from 'svelte';

	const RANKERS = gql`
		query Rankers {
			leaderboard {
				username
				score
				boardId
			}
		}
	`;

	const client = getContextClient();

	$: rankers = queryStore({
		client,
		query: RANKERS
	});

	// Sort the rankers by score in descending order
	$: sortedRankers = $rankers.data?.leaderboard.slice().sort((a: any, b: any) => b.score - a.score);

	onMount(() => {
		rankers.reexecute({ requestPolicy: 'network-only' });

		const interval = setInterval(() => {
			rankers.reexecute({ requestPolicy: 'network-only' });
		}, 5000);

		return () => clearInterval(interval);
	});
</script>

<div class="mx-auto mt-8 flex h-[calc(100vh-8rem)] w-full max-w-4xl flex-col overflow-hidden">
	<h1 class="mb-6 text-center text-4xl font-extrabold text-gray-100">Leaderboard</h1>
	<div class="flex-1 overflow-hidden bg-gray-800 px-2 py-6 shadow-xl lg:rounded-lg lg:p-6">
		{#if !$rankers.fetching || (sortedRankers?.length ?? 0) > 0}
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
					{#each sortedRankers as player, index}
						<RankerCard {player} rank={index + 1} />
					{/each}
				</div>
			</div>
		{/if}
	</div>
</div>
