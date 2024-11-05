<script lang="ts">
	import { onMount } from "svelte";

	export let value: number;
	export let canStartNewGame: boolean = true;

	let bestScore: number = 0;

	onMount(() => {
		bestScore = Number(localStorage.getItem("best"));
	});

	$: if (bestScore < value) {
		localStorage.setItem("best", String(value));
	}

	$: bestScoreValue = bestScore < value ? value : bestScore;
	$: scoreLabelAlign = value.toString().length > 3 ? 'left' : 'center';
	$: bestScoreLabelAlign = bestScoreValue.toString().length > 3 ? 'left' : 'center';
</script>

<style>
	/* Tailwind CSS is used, so no custom styles are needed here */
</style>

<div class="flex justify-between items-center w-[555px]">
	<button 
		on:click 
		class="bg-[#8f7a66] rounded-md px-5 text-[#f9f6f2] h-10 text-center border-none font-bold 
		{canStartNewGame ? 'visible' : 'invisible'}">
		New Game
	</button>
	<div class="flex flex-row items-center transition-all">
		<div class="flex flex-col bg-[#bbada0] p-2 font-bold rounded-md text-white ml-2 mb-2 min-w-16">
			<div class="text-xs text-[#eee4da] text-{scoreLabelAlign}">Score</div>
			<div class="text-2xl text-center">{value}</div>
		</div>
		<div class="flex flex-col bg-[#bbada0] p-2 font-bold rounded-md text-white ml-2 mb-2 min-w-16">
			<div class="text-xs text-[#eee4da] text-{bestScoreLabelAlign}">Best</div>
			<div class="text-2xl text-center">{bestScoreValue}</div>
		</div>
	</div>
</div>
