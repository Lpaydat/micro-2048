<script lang="ts">
    import GameListItem from '../molecules/GameListItem.svelte';
	import type { EliminationGameDetails } from '$lib/types/eliminationGame';

    export let games: Array<EliminationGameDetails> = [];
</script>

<ul class="flex flex-col gap-4 mt-8 max-w-4xl mx-auto h-full">
    {#if games.length > 0}
        {#each games as game}
            <li>
                <GameListItem data={game} />
            </li>
        {/each}
    {:else}
        <li class="text-center p-8 bg-white/90 rounded-xl shadow-lg">
            <div class="w-20 h-20 rounded-xl flex items-center justify-center mx-auto mb-4 bg-[#edc53f]">
                <i class="fas fa-gamepad text-3xl text-white"></i>
            </div>
            <h3 class="text-2xl font-bold mb-3 text-[#edc53f]">NO GAMES AVAILABLE</h3>
            <p class="text-sm bg-[#bbada0] text-white py-2 px-6 rounded-full inline-block">
                Create a new game or wait for others to host!
            </p>
        </li>
    {/if}
</ul>