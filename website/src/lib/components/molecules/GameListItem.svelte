<script lang="ts">
    import BaseListItem from './BaseListItem.svelte';
    import ActionButton from '../atoms/ActionButton.svelte';
	import type { EliminationGameDetails } from '$lib/types/eliminationGame';

    export let data: EliminationGameDetails;

    let { gameName: name, playerCount, maxPlayers, host: hostName, createdAt, totalRounds, eliminatedPerTrigger, triggerIntervalSeconds: triggerInterval } = data;

    // $: timeAgo = `${Math.floor((Date.now() - createdAt.getTime()) / (1000 * 60))} minutes ago`;
</script>

<BaseListItem>
    <div slot="left-content">
        <div class="flex items-center gap-2 mb-1">
            <h3 class="text-lg font-semibold">{name}</h3>
            <span class="text-sm bg-surface-300-600-token px-2 py-0.5 rounded-full">
                {playerCount}/{maxPlayers} players
            </span>
        </div>
        <div class="flex items-center gap-4 text-sm text-surface-700">
            <div class="flex items-center gap-2">
                <div class="w-4 h-4 bg-warning-500 rounded"></div>
                <span>{hostName}</span>
                <!-- <span>{timeAgo}</span> -->
            </div>
            <div class="flex items-center gap-2">
                <span>{totalRounds} rounds</span>
                <span>•</span>
                <span>{eliminatedPerTrigger} eliminated/trigger</span>
                <span>•</span>
                <span>{triggerInterval}s interval</span>
            </div>
        </div>
    </div>
    <div slot="right-content">
        {#if playerCount >= maxPlayers}
            <ActionButton 
                label="Full" 
                disabled={true} 
                color="disabled" 
            />
        {:else}
            <ActionButton 
                label="Join Game" 
                color="warning" 
                on:click
            />
        {/if}
    </div>
</BaseListItem>
