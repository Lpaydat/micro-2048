<script lang="ts">
	import { getContext } from 'svelte';
    import PlayerListItem from '../molecules/PlayerListItem.svelte';
    
    export let players: Array<string> = [];

    let { username }: { username: string } = getContext('player');

    // Convert simple string array to object array with default values
    $: playerObjects = players.map((name, index) => ({
        name,
        isHost: index === 0, // Assuming first player is host
        isYou: name === username,
        joinedAt: new Date() // You might want to pass this as actual data
    }));
</script>

<div class="flex flex-col gap-4 mt-8 max-w-4xl mx-auto">
    <ul class="flex flex-col gap-4">
        {#if playerObjects.length > 0}
            {#each playerObjects as player}
                <li>
                    <PlayerListItem 
                        playerName={player.name}
                        isHost={player.isHost}
                        isYou={player.isYou}
                        joinedAt={player.joinedAt}
                    />
                </li>
            {/each}
        {:else}
            <li class="text-center p-8 bg-white/90 rounded-xl shadow-lg">
                <div class="w-20 h-20 rounded-xl flex items-center justify-center mx-auto mb-4 bg-[#edc53f]">
                    <i class="fas fa-user-group text-3xl text-white"></i>
                </div>
                <h3 class="text-2xl font-bold mb-3 text-[#edc53f]">NO PLAYERS YET</h3>
                <p class="text-sm bg-[#bbada0] text-white py-2 px-6 rounded-full inline-block">
                    Waiting for other players to join...
                </p>
            </li>
        {/if}
    </ul>
</div>
