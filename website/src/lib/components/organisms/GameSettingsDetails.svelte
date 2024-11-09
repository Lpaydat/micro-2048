<script lang="ts">
    import type { EliminationGameDetails } from '$lib/types/eliminationGame';
	import TimeAgo from '../atoms/TimeAgo.svelte';

    export let data: EliminationGameDetails;
    export let numberLabel: string = 'Players';
    export let numberA: number | undefined = 1;
    export let numberB: number;

    // Store the initial data in a local variable
    let initialData: EliminationGameDetails = { ...data };

    // Reactive statement to update only numberA and numberB
    $: updatedData = {
        ...initialData,
        gameName: initialData.gameName || data.gameName,
        host: initialData.host || data.host,
        createdTime: initialData.createdTime || data.createdTime,
        totalRounds: initialData.totalRounds || data.totalRounds,
        eliminatedPerTrigger: initialData.eliminatedPerTrigger || data.eliminatedPerTrigger,
        triggerIntervalSeconds: initialData.triggerIntervalSeconds || data.triggerIntervalSeconds,
        numberA,
        numberB
    };
</script>

<div class="game-details bg-[#faf8ef] p-4 mt-8 rounded-lg shadow-md max-w-2xl mx-auto">
    <div class="flex flex-wrap items-center gap-4">
        <div class="flex items-center gap-2">
            <span class="font-bold text-[#776e65]">{updatedData.gameName}</span>
            <div class="bg-[#edc403] text-[#776e65] px-2 py-0.5 rounded-full text-sm font-semibold">
                {numberLabel}: {numberA}/{numberB}
            </div>
        </div>

        <div class="flex flex-wrap items-center gap-4 text-sm">
            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Host</span>
                <span class="font-bold text-[#776e65]">{updatedData.host}</span>
            </div>

            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Created</span>
                <span class="font-bold text-[#776e65]"><TimeAgo time={updatedData.createdTime} /></span>
            </div>

            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Rounds</span>
                <span class="font-bold text-[#776e65]">{updatedData.totalRounds}</span>
            </div>

            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Eliminated</span>
                <span class="font-bold text-[#776e65]">{updatedData.eliminatedPerTrigger} per trigger</span>
            </div>

            <div class="flex items-center gap-2">
                <span class="text-[#bbada0]">Interval</span>
                <span class="font-bold text-[#776e65]">{updatedData.triggerIntervalSeconds}s</span>
            </div>
        </div>
    </div>
</div>

<style>
    .game-details {
        font-family: "Clear Sans", "Helvetica Neue", Arial, sans-serif;
    }
</style> 