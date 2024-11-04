<script lang="ts">
    import ActionButton from '../atoms/ActionButton.svelte';

    export let gameName: string;
    export let playerCount: number;
    export let maxPlayers: number;
    export let hostName: string;
    export let createdAt: Date;
    export let totalRounds: number;
    export let eliminatedPerRound: number;

    // Format the created time to show how long ago the game was created
    $: timeAgo = `${Math.floor((Date.now() - createdAt.getTime()) / (1000 * 60))} minutes ago`;
</script>

<div class="flex items-center justify-between p-4 bg-surface-50-900-token rounded-lg hover:bg-surface-100-800-token border-4 hover:border-warning-900 transition-all">
    <div class="flex-1">
        <div class="flex items-center gap-2 mb-1">
            <h3 class="text-lg font-semibold">{gameName}</h3>
            <span class="text-sm bg-surface-300-600-token px-2 py-0.5 rounded-full">
                {playerCount}/{maxPlayers} players
            </span>
        </div>
        <div class="flex items-center gap-4 text-sm text-surface-700">
            <div class="flex items-center gap-2">
                <div class="w-4 h-4 bg-warning-500 rounded"></div>
                <span>{hostName}</span>
                <span>{timeAgo}</span>
            </div>
            <div class="flex items-center gap-2">
                <span>{totalRounds} rounds</span>
                <span>•</span>
                <span>{eliminatedPerRound} eliminated/round</span>
            </div>
        </div>
    </div>
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
