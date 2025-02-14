<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { boardSize } from '$lib/stores/gameStore';
	import { getBoardId, setBoardId } from '$lib/stores/boardId';
	import { userStore } from '$lib/stores/userStore';
	import UsernameBadge from '../atoms/UsernameBadge.svelte';
	import { newGameBoard } from '$lib/game/newGameBoard';
	import { addShards, getRandomShard, getShards } from '$lib/stores/shards';
	import { gql } from 'urql';
	import { getClient } from '$lib/client';
	import { queryStore } from '@urql/svelte';
	import { getBoard } from '$lib/graphql/queries/getBoard';

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

	const LEADERBOARD = gql`
		query Leaderboard {
			leaderboard {
				leaderboardId
				shardIds
			}
		}
	`;

	const leaderboardId = $derived($page.url.searchParams.get('leaderboardId') ?? '');
	const isOwner = $derived(player === $userStore.username);

	const leaderboardClient = $derived(getClient(leaderboardId, true));
	const playerClient = getClient($userStore.chainId, true);

	const leaderboard = $derived(
		queryStore({
			client: leaderboardClient,
			query: LEADERBOARD
		})
	);

	const board = $derived(getBoard(playerClient));

	// Size configurations
	const sizeConfig = {
		xs: { width: 296, buttonHeight: 5, fontSize: 'text-sm', scoreSize: 'text-md' },
		sm: { width: 370, buttonHeight: 6, fontSize: 'text-base', scoreSize: 'text-lg' },
		md: { width: 460, buttonHeight: 9, fontSize: 'text-xl', scoreSize: 'text-2xl' },
		lg: { width: 555, buttonHeight: 10, fontSize: 'text-2xl', scoreSize: 'text-2xl' }
	};

	let newGameAt = $state(Date.now());
	let isNewGameCreated = $state(false);
	let showCooldownMessage = $state(false);

	// Mutation functions
	const newSingleGame = async () => {
		// Prevent creating more than 1 game per 10 seconds
		if (Date.now() - newGameAt < 10000) {
			showCooldownMessage = true;
			setTimeout(() => {
				showCooldownMessage = false;
			}, 3000);
			return;
		}
		if (!canStartNewGame || !leaderboardId || !$userStore.username) return;

		const shardId = await getRandomShard(leaderboardId, $userStore.username);
		if (!shardId) return;

		await newGameBoard(leaderboardId, shardId, newGameAt.toString());
		isNewGameCreated = true;
	};

	$effect(() => {
		if ($leaderboard.data?.leaderboard?.shardIds?.length) {
			const shards = getShards(leaderboardId);
			if (!shards?.length) {
				addShards(leaderboardId, $leaderboard.data?.leaderboard?.shardIds);
			}
		}
	});

	onMount(() => {
		const interval = setInterval(() => {
			board?.reexecute({ requestPolicy: 'network-only' });
			if (
				isNewGameCreated &&
				$board?.data?.board?.boardId &&
				newGameAt &&
				$board?.data?.board?.createdAt &&
				Math.abs($board?.data?.board?.createdAt - newGameAt) < 10000 &&
				$board?.data?.board?.leaderboardId === leaderboardId
			) {
				newGameAt = Date.now();
				isNewGameCreated = false;
				const url = new URL('/game', window.location.origin);
				url.searchParams.set('boardId', $board?.data?.board?.boardId);
				url.searchParams.set('leaderboardId', leaderboardId);

				setBoardId($board.data?.board?.boardId, leaderboardId);
				goto(url.toString(), { replaceState: false });
			}
		}, 1000);

		return () => clearInterval(interval);
	});

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
				{#if showCooldownMessage}
					<span>Wait a sec!</span>
				{:else}
					<span>New Game</span>
				{/if}
			</button>
		</div>
	{:else}
		<UsernameBadge username={player} fontSize={currentSize.fontSize} />
	{/if}
	<div class="flex flex-row items-center transition-all">
		{#if player && canStartNewGame && !isOwner}
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
