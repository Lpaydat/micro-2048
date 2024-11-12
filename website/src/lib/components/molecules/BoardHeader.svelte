<script lang="ts">
	import { getContextClient, mutationStore, gql } from "@urql/svelte";
	import { onMount } from "svelte";

	export let player: string;
	export let value: number;
	export let canStartNewGame: boolean = true;
	export let showBestScore: boolean = true;
	export let boardId: string | undefined = undefined;

	let bestScore: number = 0;
	let client = getContextClient();

	const NEW_BOARD = gql`
		mutation NewBoard($seed: Int!, $player: String!) {
		newBoard(seed: $seed, player: $player)
		}
	`;

	// Mutation functions
	const newGameMutation = ({ seed }: { seed: number }) => {
		mutationStore({
			client,
			query: NEW_BOARD,
			variables: { seed, player },
		});
	};

	const newSingleGame = () => {
		if (!canStartNewGame) return;
		boardId = Math.floor(Math.random() * 1000000).toString();
		newGameMutation({ seed: parseInt(boardId) });
	}

	onMount(() => {
		bestScore = Number(localStorage.getItem("best"));

		setTimeout(() => {
			newSingleGame();
		}, 50);
	});

	$: if (bestScore < value) {
		localStorage.setItem("best", String(value));
	}

	$: bestScoreValue = bestScore < value ? value : bestScore;
	$: scoreLabelAlign = value.toString().length > 3 ? 'left' : 'center';
	$: bestScoreLabelAlign = bestScoreValue.toString().length > 3 ? 'left' : 'center';
</script>

<div class="flex justify-between items-center w-[555px]">
	{#if canStartNewGame}
		<button 
			on:click={newSingleGame}
			class="bg-[#8f7a66] rounded-md px-5 text-[#f9f6f2] h-10 text-center border-none font-bold 
			{canStartNewGame ? 'visible' : 'invisible'}">
			New Game
		</button>
	{:else}
		<div class="flex items-center justify-center text-2xl text-[#f67c5f] font-bold" style="font-family: 'Clear Sans', 'Arial', sans-serif;">
			{player}
		</div>
	{/if}
	<div class="flex flex-row items-center transition-all">
		<div class="flex flex-col bg-[#bbada0] p-2 font-bold rounded-md text-white ml-2 mb-2 min-w-16">
			<div class="text-xs text-[#eee4da] text-{scoreLabelAlign}">Score</div>
			<div class="text-2xl text-center">{value}</div>
		</div>
		{#if showBestScore}
			<div class="flex flex-col bg-[#bbada0] p-2 font-bold rounded-md text-white ml-2 mb-2 min-w-16">
				<div class="text-xs text-[#eee4da] text-{bestScoreLabelAlign}">Best</div>
				<div class="text-2xl text-center">{bestScoreValue}</div>
			</div>
		{/if}
	</div>
</div>
