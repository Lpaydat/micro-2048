<script lang="ts">
	import type { EliminationGameDetails } from '$lib/types/eliminationGame';
	import TimeAgo from '../atoms/TimeAgo.svelte';

	interface Props
		extends Pick<
			EliminationGameDetails,
			| 'gameName'
			| 'host'
			| 'createdTime'
			| 'totalRounds'
			| 'eliminatedPerTrigger'
			| 'triggerIntervalSeconds'
		> {
		numberLabel?: string;
		numberA?: number;
		numberB: number;
	}

	let { numberLabel = 'Players', numberA = 1, numberB, ...rest }: Props = $props();

	// Initialize variables directly with the values from rest
	const gameName = rest.gameName;
	const host = rest.host;
	const createdTime = rest.createdTime;
	const totalRounds = rest.totalRounds;
	const eliminatedPerTrigger = rest.eliminatedPerTrigger;
	const triggerIntervalSeconds = rest.triggerIntervalSeconds;
</script>

<div class="game-details m-2 max-w-2xl rounded-lg bg-[#faf8ef] p-4 shadow-md sm:mx-auto sm:mt-8">
	<div class="flex flex-wrap items-center gap-4">
		<div class="flex items-center gap-2">
			<span class="font-bold text-[#776e65]">{gameName}</span>
			<div class="rounded-full bg-[#edc403] px-2 py-0.5 text-sm font-semibold text-[#776e65]">
				{numberLabel}: {numberA}/{numberB}
			</div>
		</div>

		<div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
			<div class="flex items-center gap-2">
				<span class="text-[#bbada0]">Creator</span>
				<span class="font-bold text-[#776e65]">{host}</span>
			</div>

			<div class="flex items-center gap-2">
				<span class="text-[#bbada0]">Created</span>
				<span class="font-bold text-[#776e65]"><TimeAgo time={createdTime} /></span>
			</div>

			<div class="flex items-center gap-2">
				<span class="text-[#bbada0]">Rounds</span>
				<span class="font-bold text-[#776e65]">{totalRounds}</span>
			</div>

			<div class="flex items-center gap-2">
				<span class="text-[#bbada0]">Eliminated</span>
				<span class="font-bold text-[#776e65]">{eliminatedPerTrigger} per trigger</span>
			</div>

			<div class="flex items-center gap-2">
				<span class="text-[#bbada0]">Interval</span>
				<span class="font-bold text-[#776e65]">{triggerIntervalSeconds}s</span>
			</div>
		</div>
	</div>
</div>

<style>
	.game-details {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
	}
</style>
