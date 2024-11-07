<script lang="ts">
    import Smile from 'lucide-svelte/icons/smile';
    import BaseListItem from './BaseListItem.svelte';
    
    export let playerName: string;
    export let isHost: boolean = false;
    export let isYou: boolean = false;
    export let joinedAt: Date;

    // Format the joined time to show how long ago the player joined
    $: timeAgo = `${Math.floor((Date.now() - joinedAt.getTime()) / (1000 * 60))} minutes ago`;
</script>

<BaseListItem>
    <div slot="left-content">
        <div class="flex items-center justify-between gap-2 mb-1">
            <div class="flex items-center gap-2">
                <h3 class="text-lg font-semibold">{playerName}</h3>
                {#if isHost}
                    <span class="text-xs font-semibold bg-orange-500 px-2 py-0.5 rounded-full text-white">
                        HOST
                    </span>
                {:else if isYou}
                    <Smile size={20} />
                {/if}
            </div>
            <div class="flex items-center gap-2 text-sm text-surface-700">
                <div class="w-4 h-4 bg-warning-500 rounded"></div>
                <span>{timeAgo}</span>
            </div>
        </div>
    </div>
</BaseListItem> 
