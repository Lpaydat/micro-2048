<script lang="ts">
	import { getContext } from 'svelte';
	import { goto } from '$app/navigation';
	import { gql, getContextClient, mutationStore } from '@urql/svelte';
    import BaseListItem from './BaseListItem.svelte';
    import ActionButton from '../atoms/ActionButton.svelte';
	import type { EliminationGameDetails } from '$lib/types/eliminationGame';

    export let data: EliminationGameDetails;

    let {
        gameId,
        gameName,
        playerCount,
        maxPlayers,
        host,
        createdTime,
        totalRounds,
        eliminatedPerTrigger,
        triggerIntervalSeconds
    } = data;

    $: timeAgo = `${Math.floor((Date.now() - new Date(createdTime).getTime()) / (1000 * 60))} minutes ago`;

    const client = getContextClient();
    const { username }: { username: string } = getContext('player');
    let loading = false;

    const JOIN_GAME = gql`
        mutation JoinEliminationGame($player: String!, $gameId: String!) {
            joinEliminationGame(player: $player, gameId: $gameId)
        }
    `;

    const joinGame = (gameId: string) => {
        loading = true;
        mutationStore({
            client, query: JOIN_GAME, variables: { player: username, gameId }
        });
    }

    const enterGame = (gameId: string) => {
        goto(`/elimination/${gameId}`);
    }
</script>

<BaseListItem>
    <div slot="left-content">
        <div class="flex items-center gap-2 mb-1">
            <h3 class="text-lg font-semibold">{gameName}</h3>
            <span class="text-sm bg-surface-300-600-token px-2 py-0.5 rounded-full">
                {playerCount}/{maxPlayers} players
            </span>
        </div>
        <div class="flex items-center gap-4 text-sm text-surface-700">
            <div class="flex items-center gap-2">
                <div class="w-4 h-4 bg-warning-500 rounded"></div>
                <span>{host}</span>
                <span>{timeAgo}</span>
            </div>
            <div class="flex items-center gap-2">
                <span>{totalRounds} rounds</span>
                <span>•</span>
                <span>{eliminatedPerTrigger} eliminated/trigger</span>
                <span>•</span>
                <span>{triggerIntervalSeconds}s interval</span>
            </div>
        </div>
    </div>
    <div slot="right-content">
        {#if username === host}
            <ActionButton 
                label="Enter Game" 
                color="important"
                on:click={() => enterGame(gameId)} 
            />
        {:else if playerCount >= maxPlayers}
            <ActionButton 
                label="Full" 
                disabled={true} 
                color="disabled" 
            />
        {:else}
            <ActionButton 
                label="Join Game" 
                color="warning" 
                loading={loading}
                on:click={() => joinGame(gameId)}
            />
        {/if}
    </div>
</BaseListItem>
