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
        query: RANKERS,
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

<div class="w-full mx-auto mt-8 max-w-4xl">
    <h1 class="text-4xl font-extrabold mb-6 text-center text-gray-100">Leaderboard</h1>
    <div class="bg-gray-800 px-2 py-6 lg:p-6 lg:rounded-lg shadow-xl">
        {#if !$rankers.fetching}
            <div class="space-y-4">
                <!-- Header Row -->
                <div class="flex justify-between items-center text-xs lg:text-base px-4 text-gray-400 font-semibold border-b border-gray-700 pb-2">
                    <span class="w-1/12 text-left">#</span>
                    <span class="w-4/12">Username</span>
                    <span class="w-3/12">Board ID</span>
                    <span class="w-4/12 text-right">Score</span>
                </div>
                <!-- Player Rows -->
                {#each sortedRankers as player, index}
                    <RankerCard {player} rank={index + 1} />
                {/each}
            </div>
        {/if}
    </div>
</div>