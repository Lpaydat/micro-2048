<script lang="ts">
	import { getContextClient } from '@urql/svelte';
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { newGame } from '$lib/graphql/mutations/newBoard';
	import { hashSeed } from '$lib/utils/random';
	import { boardSize, setGameCreationStatus } from '$lib/stores/gameStore';
	import { getBoardId, setBoardId } from '$lib/stores/boardId';
	import { userStore } from '$lib/stores/userStore';
	import UsernameBadge from '../atoms/UsernameBadge.svelte';
	import { should } from 'vitest';

	interface Props {
		player: string;
		score: number;
		bestScore?: number;
		canStartNewGame?: boolean;
		showBestScore?: boolean;
		boardId?: string | undefined;
	}

	let {
		player,
		score,
		bestScore = 0,
		canStartNewGame = true,
		showBestScore = true,
		boardId = $bindable()
	}: Props = $props();

	const client = getContextClient();
	const leaderboardId = $derived($page.url.searchParams.get('leaderboardId') ?? '');
	const isOwner = $derived(player === $userStore.username);

	// Size configurations
	const sizeConfig = {
		xs: { width: 296, buttonHeight: 5, fontSize: 'text-sm', scoreSize: 'text-md' },
		sm: { width: 370, buttonHeight: 6, fontSize: 'text-base', scoreSize: 'text-lg' },
		md: { width: 460, buttonHeight: 9, fontSize: 'text-xl', scoreSize: 'text-2xl' },
		lg: { width: 555, buttonHeight: 10, fontSize: 'text-2xl', scoreSize: 'text-2xl' }
	};

	// Mutation functions
	const newSingleGame = async () => {
		if (!canStartNewGame || !$userStore.username) return;
		const seed = Math.floor(Math.random() * 10_000_000).toString();
		const timestamp = Date.now().toString();

		boardId = (await hashSeed(seed, $userStore.username, timestamp)).toString();
		setBoardId(boardId, leaderboardId);
		setGameCreationStatus(true);
		newGame(client, seed, timestamp, leaderboardId);

		const url = new URL($page.url);
		url.searchParams.set('boardId', boardId);
		goto(url.toString(), { replaceState: true });
	};

	onMount(() => {
		setTimeout(() => {
			if (!getBoardId(leaderboardId)) {
				newSingleGame();
			}
		}, 100);
	});

	const shouldShowBestScore = $derived(showBestScore && canStartNewGame);
	const scoreLabelAlign = $derived(score.toString().length > 3 ? 'left' : 'center');
	const bestScoreLabelAlign = $derived(bestScore.toString().length > 3 ? 'left' : 'center');
	const currentSize = $derived(sizeConfig[$boardSize]);
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
		<UsernameBadge username={player} fontSize={currentSize.fontSize} />
	{/if}
	<div class="flex flex-row items-center transition-all">
		{#if player && canStartNewGame && !isOwner && !shouldShowBestScore}
			<UsernameBadge username={player} fontSize={currentSize.fontSize} />
		{/if}
		<div class="mb-2 ml-2 flex min-w-16 flex-col rounded-md bg-[#bbada0] p-2 font-bold text-white">
			<div class="text-xs text-[#eee4da] text-{scoreLabelAlign}">Score</div>
			<div class="{currentSize.scoreSize} text-center">{score}</div>
		</div>
		{#if shouldShowBestScore && isOwner}
			<div
				class="mb-2 ml-2 flex min-w-16 flex-col rounded-md bg-[#bbada0] p-2 font-bold text-white"
			>
				<div class="text-xs text-[#eee4da] text-{bestScoreLabelAlign}">Best</div>
				<div class="{currentSize.scoreSize} text-center">{bestScore}</div>
			</div>
		{/if}
	</div>
</div>
