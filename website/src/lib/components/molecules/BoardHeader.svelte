<script lang="ts">
	import { getContextClient } from '@urql/svelte';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { newGame } from '$lib/graphql/mutations/newBoard';
	import { hashSeed } from '$lib/utils/random';
	import { setGameCreationStatus } from '$lib/stores/gameStore';
	import { userStore } from '$lib/stores/userStore';
	import { getBoardId, setBoardId } from '$lib/stores/boardId';

	interface Props {
		player: string;
		value: number;
		canStartNewGame?: boolean;
		showBestScore?: boolean;
		boardId?: string | undefined;
		size?: 'sm' | 'md' | 'lg';
	}

	let {
		player,
		value,
		canStartNewGame = true,
		showBestScore = true,
		boardId = $bindable(),
		size = 'lg'
	}: Props = $props();

	let bestScore = $state(0);

	const client = getContextClient();
	const leaderboardId = $page.url.searchParams.get('leaderboardId') ?? '';

	// Size configurations
	const sizeConfig = {
		sm: { width: 370, buttonHeight: 6, fontSize: 'text-base', scoreSize: 'text-lg' },
		md: { width: 460, buttonHeight: 9, fontSize: 'text-xl', scoreSize: 'text-2xl' },
		lg: { width: 555, buttonHeight: 10, fontSize: 'text-2xl', scoreSize: 'text-2xl' }
	};

	// Mutation functions
	const newSingleGame = async () => {
		if (!canStartNewGame) return;
		const seed = Math.floor(Math.random() * 10_000_000).toString();
		const timestamp = Date.now().toString();

		boardId = (await hashSeed(seed, player, timestamp)).toString();
		setBoardId(boardId, leaderboardId);
		setGameCreationStatus(true);
		newGame(client, seed, timestamp, leaderboardId);

		const url = new URL($page.url);
		url.searchParams.set('boardId', boardId);
		goto(url.toString(), { replaceState: true });
	};

	onMount(() => {
		bestScore = Number(localStorage.getItem('highestScore'));

		setTimeout(() => {
			if (!getBoardId(leaderboardId)) {
				newSingleGame();
			}
		}, 50);
	});

	$effect(() => {
		if (bestScore < value && player === $userStore.username) {
			localStorage.setItem('highestScore', String(value));
		}
	});

	const bestScoreValue = $derived(bestScore < value ? value : bestScore);
	const shouldShowBestScore = $derived(showBestScore && canStartNewGame);
	const scoreLabelAlign = $derived(value.toString().length > 3 ? 'left' : 'center');
	const bestScoreLabelAlign = $derived(bestScoreValue.toString().length > 3 ? 'left' : 'center');
	const currentSize = $derived(sizeConfig[size]);
</script>

<div class="flex items-center justify-between" style="width: {currentSize.width}px">
	{#if canStartNewGame}
		<div class="flex items-center gap-2">
			<button
				onclick={newSingleGame}
				class="text-md rounded-md border-none bg-[#8f7a66] px-2 py-2 text-center font-bold text-[#f9f6f2] md:px-4 md:text-xl
				{canStartNewGame ? 'visible' : 'invisible'}"
			>
				<span>New Game</span>
			</button>
		</div>
	{:else}
		<div
			class="flex items-center justify-center truncate font-bold text-[#f67c5f] {currentSize.fontSize}"
			style="font-family: 'Clear Sans', 'Arial', sans-serif;"
		>
			{player}
		</div>
	{/if}
	<div class="flex flex-row items-center transition-all">
		<div class="mb-2 ml-2 flex min-w-16 flex-col rounded-md bg-[#bbada0] p-2 font-bold text-white">
			<div class="text-xs text-[#eee4da] text-{scoreLabelAlign}">Score</div>
			<div class="{currentSize.scoreSize} text-center">{value}</div>
		</div>
		{#if shouldShowBestScore}
			<div
				class="mb-2 ml-2 flex min-w-16 flex-col rounded-md bg-[#bbada0] p-2 font-bold text-white"
			>
				<div class="text-xs text-[#eee4da] text-{bestScoreLabelAlign}">Best</div>
				<div class="{currentSize.scoreSize} text-center">{bestScoreValue}</div>
			</div>
		{/if}
	</div>
</div>
