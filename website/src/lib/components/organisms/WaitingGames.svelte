<script lang="ts">
	import { getContext, onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import type { EliminationGameDetails } from '$lib/types/eliminationGame';
    import GameListItem from '../molecules/GameListItem.svelte';

    let games: Array<EliminationGameDetails> = [];

    const GET_WAITING_GAMES = gql`
        query GetWaitingGames {
            waitingRooms {
                gameId
                chainId
                gameName
                host
                players
                maxPlayers
                totalRounds
                triggerIntervalSeconds
                eliminatedPerTrigger
                createdTime
            }
        }
    `;

    const client = getContextClient();
    const { username }: { username: string } = getContext('player');

    $: waitingGames = queryStore({
        client,
        query: GET_WAITING_GAMES,
    });

    onMount(() => {
        waitingGames.reexecute({ requestPolicy: 'network-only' }); // Initial fetch
        const interval = setInterval(() => {
            waitingGames.reexecute({ requestPolicy: 'network-only' }); // Fetch every second
        }, 1000); // Fetch every second

        return () => {
            clearInterval(interval); // Cleanup interval on component destroy
        };
    });

    let initialFetch = true;
    $: if (initialFetch && !$waitingGames.fetching) {
        initialFetch = false;
    }

    $: {
        games = ($waitingGames.data?.waitingRooms ?? []).map((game: any) => {
            if (game.players.includes(username)) {
                goto(`/elimination/${game.gameId}`);
            }

            return {
                ...game,
                playerCount: game.players.length,
                createdTime: new Date(parseInt(game.createdTime))
            }
        });
    }
</script>

<ul class="flex flex-col gap-4 mt-8 max-w-4xl mx-auto h-full">
    {#if initialFetch}
        <li class="text-center p-8">
            <div class="font-bold text-2xl text-[#776e65]">Loading...</div> <!-- Using 2048 game's yellow text color -->
        </li>
    {:else if games.length > 0}
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