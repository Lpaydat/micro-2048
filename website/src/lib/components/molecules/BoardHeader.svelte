<script lang="ts">
	import { getContextClient } from "@urql/svelte";
	import { onMount } from "svelte";
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { newGame } from "$lib/graphql/mutations/newBoard";
	import { hashSeed } from "$lib/utils/random";

	export let player: string;
	export let value: number;
	export let canStartNewGame: boolean = true;
	export let showBestScore: boolean = true;
	export let boardId: string | undefined = undefined;
	export let size: 'sm' | 'md' | 'lg' = 'lg';

	let bestScore: number = 0;
	let client = getContextClient();

	// Size configurations
	const sizeConfig = {
		sm: { width: 370, buttonHeight: 6, fontSize: 'text-base', scoreSize: 'text-lg' },
		md: { width: 460, buttonHeight: 9, fontSize: 'text-xl', scoreSize: 'text-2xl' },
		lg: { width: 555, buttonHeight: 10, fontSize: 'text-2xl', scoreSize: 'text-2xl' }
	};

	// Mutation functions
	const newGameMutation = (seed: string, timestamp: string) => newGame(client, seed, timestamp);

	const newSingleGame = async () => {
		if (!canStartNewGame) return;
		const seed = Math.floor(Math.random() * 10_000_000).toString();
		const timestamp = Date.now().toString();

		boardId = (await hashSeed(seed, player, timestamp)).toString();
		localStorage.setItem('boardId', boardId);

		newGameMutation(seed, timestamp);

		const url = new URL($page.url);
		url.searchParams.delete('boardId');
		goto(url.toString(), { replaceState: true });
	}

	onMount(() => {
		bestScore = Number(localStorage.getItem("highestScore"));

		setTimeout(() => {
			if (!localStorage.getItem('boardId')) {
				newSingleGame();
			}
		}, 50);
	});

	$: if (bestScore < value) {
		localStorage.setItem("highestScore", String(value));
	}

	$: bestScoreValue = bestScore < value ? value : bestScore;
	$: isSpecBoard = !!$page.url.searchParams.get('boardId');
	$: scoreLabelAlign = value.toString().length > 3 ? 'left' : 'center';
	$: bestScoreLabelAlign = bestScoreValue.toString().length > 3 ? 'left' : 'center';
	$: currentSize = sizeConfig[size];
</script>

<div class="flex justify-between items-center" style="width: {currentSize.width}px">
	{#if canStartNewGame}
		<button 
			on:click={newSingleGame}
			class="bg-[#8f7a66] rounded-md px-5 text-[#f9f6f2] text-center border-none font-bold 
			{canStartNewGame ? 'visible' : 'invisible'}"
			style="height: {currentSize.buttonHeight * 4}px"
		>
			<span class="{currentSize.fontSize}">New Game</span>
		</button>
	{:else}
		<div 
			class="flex items-center justify-center text-[#f67c5f] font-bold truncate {currentSize.fontSize}" 
			style="font-family: 'Clear Sans', 'Arial', sans-serif;"
		>
			{player}
		</div>
	{/if}
	<div class="flex flex-row items-center transition-all">
		<div class="flex flex-col bg-[#bbada0] p-2 font-bold rounded-md text-white ml-2 mb-2 min-w-16">
			<div class="text-xs text-[#eee4da] text-{scoreLabelAlign}">Score</div>
			<div class="{currentSize.scoreSize} text-center">{value}</div>
		</div>
		{#if showBestScore && !isSpecBoard}
			<div class="flex flex-col bg-[#bbada0] p-2 font-bold rounded-md text-white ml-2 mb-2 min-w-16">
				<div class="text-xs text-[#eee4da] text-{bestScoreLabelAlign}">Best</div>
				<div class="{currentSize.scoreSize} text-center">{bestScoreValue}</div>
			</div>
		{/if}
	</div>
</div>
