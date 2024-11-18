<script lang="ts">
	import { goto } from "$app/navigation";

    export let isRoundEnded: boolean;
    export let countdown: number;
    export let status: string;
</script>

<div class="h-32 flex items-center justify-center pt-10">
    {#if !isRoundEnded}
        <div class="text-[#776E65] text-7xl font-bold game-font text-center">
            {countdown >= 0 ? countdown : '...'}
        </div>
    {:else if status === 'Ended'}
        <button
            class="btn variant-soft-surface text-[#F67C5F] text-3xl font-bold game-font text-center"
            on:click={() => goto('/elimination')}
        >
            Lobby
        </button>
    {:else}
        <button
            class="btn variant-soft-surface text-[#F67C5F] text-3xl font-bold game-font text-center"
            on:click
        >
            Next Round
        </button>
    {/if}
</div>