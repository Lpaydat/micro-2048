<script lang="ts">
    import type { EliminationGameDetails } from '$lib/types/eliminationGame';

    export let data: EliminationGameDetails;
    export let numberLabel: string = 'Players';
    export let numberA: number | undefined = 1;
    export let numberB: number;

    $: timeAgo = `${Math.floor((Date.now() - new Date(data.createdAt).getTime()) / (1000 * 60))} minutes ago`;
</script>

<div class="game-details bg-[#faf8ef] p-4 mt-8 rounded-lg shadow-md max-w-2xl mx-auto">
    <div class="flex flex-wrap items-center gap-4">
        <div class="flex items-center gap-2">
            <span class="font-bold text-[#776e65]">{data.gameName}</span>
            <div class="bg-[#edc403] text-[#776e65] px-2 py-0.5 rounded-full text-sm font-semibold">
                {numberLabel}: {numberA}/{numberB}
            </div>
        </div>

        <div class="flex flex-wrap items-center gap-4 text-sm">
            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Host</span>
                <span class="font-bold text-[#776e65]">{data.host}</span>
            </div>

            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Created</span>
                <span class="font-bold text-[#776e65]">{timeAgo}</span>
            </div>

            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Rounds</span>
                <span class="font-bold text-[#776e65]">{data.totalRounds}</span>
            </div>

            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Eliminated</span>
                <span class="font-bold text-[#776e65]">{data.eliminatedPerTrigger} per trigger</span>
            </div>

            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Interval</span>
                <span class="font-bold text-[#776e65]">{data.triggerIntervalSeconds}s</span>
            </div>
        </div>
    </div>
</div>

<style>
    .game-details {
        font-family: "Clear Sans", "Helvetica Neue", Arial, sans-serif;
    }
</style> 