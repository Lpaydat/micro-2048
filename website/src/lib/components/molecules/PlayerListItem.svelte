<script lang="ts">
    import BaseListItem from './BaseListItem.svelte';
    
    export let playerName: string;
    export let isHost: boolean = false;
    export let joinedAt: Date;

    // Format the joined time to show how long ago the player joined
    $: timeAgo = `${Math.floor((Date.now() - joinedAt.getTime()) / (1000 * 60))} minutes ago`;
</script>

<BaseListItem>
    <div slot="left-content">
        <div class="flex items-center gap-2 mb-1">
            <h3 class="text-lg font-semibold">{playerName}</h3>
            {#if isHost}
                <span class="text-sm bg-warning-500 px-2 py-0.5 rounded-full text-white">
                    Host
                </span>
            {/if}
        </div>
        <div class="flex items-center gap-4 text-sm text-surface-700">
            <div class="flex items-center gap-2">
                <div class="w-4 h-4 bg-warning-500 rounded"></div>
                <span>{timeAgo}</span>
            </div>
        </div>
    </div>
</BaseListItem> 