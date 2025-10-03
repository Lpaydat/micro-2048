<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { boardSize } from '$lib/stores/gameStore';
	import { getBoardId, setBoardId } from '$lib/stores/boardId';
	import { userStore } from '$lib/stores/userStore';
	import UsernameBadge from '../atoms/UsernameBadge.svelte';
	import { newGameBoard } from '$lib/game/newGameBoard';
	import { addShards, getShards } from '$lib/stores/shards';
	import { gql } from 'urql';
	import { getClient } from '$lib/client';
	import { queryStore } from '@urql/svelte';
	import { getBoard, getBoards } from '$lib/graphql/queries/getBoard';

	interface Props {
		player: string;
		score: number;
		bestScore?: number;
		canStartNewGame?: boolean;
		showBestScore?: boolean;
		boardId?: string | undefined;
		isCreating?: boolean;
	}

	let {
		player,
		score,
		bestScore = 0,
		canStartNewGame = true,
		showBestScore = true,
		boardId = $bindable(),
		isCreating = $bindable(false)
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
	const playerClient = $derived(getClient($userStore.chainId, true));

	const leaderboard = $derived(
		queryStore({
			client: leaderboardClient,
			query: LEADERBOARD
		})
	);

	const board = $derived(getBoard(playerClient));
	const boards = $derived(getBoards(playerClient, 5)); // Only fetch last 5 boards

	// If board query returns null, find latest board from boards array
	const latestBoard = $derived.by(() => {
		if ($board?.data?.board) return $board.data.board;

		// Fallback: find most recent board from boards array
		const allBoards = $boards?.data?.boards || [];
		if (allBoards.length === 0) return null;

		return allBoards.sort((a: any, b: any) => {
			return parseInt(b.createdAt || '0') - parseInt(a.createdAt || '0');
		})[0];
	});

	// Size configurations
	const sizeConfig = {
		xs: { width: 296, buttonHeight: 5, fontSize: 'text-sm', scoreSize: 'text-md' },
		sm: { width: 370, buttonHeight: 6, fontSize: 'text-base', scoreSize: 'text-lg' },
		md: { width: 460, buttonHeight: 9, fontSize: 'text-xl', scoreSize: 'text-2xl' },
		lg: { width: 555, buttonHeight: 10, fontSize: 'text-2xl', scoreSize: 'text-2xl' }
	};

	let newGameAt = $state(Date.now());
	let isNewGameCreated = $state(false);
	let boardCreationStartTime = $state<number | null>(null);

	// Check if button should be disabled (creating board OR within 5 seconds of last creation)
	const isCreatingBoard = $derived(
		isNewGameCreated || !!(boardCreationStartTime && Date.now() - boardCreationStartTime < 5000)
	);

	// Update parent's isCreating binding
	$effect(() => {
		isCreating = !!isCreatingBoard;
	});

	// Mutation functions
	const newSingleGame = async () => {
		if (!canStartNewGame || !$userStore.username) return;
		if (isCreatingBoard) return; // Prevent multiple clicks

		// If no leaderboardId, navigate to leaderboard selection page
		if (!leaderboardId) {
			goto('/events');
			return;
		}

		try {
			boardCreationStartTime = Date.now();
			newGameAt = Date.now();

			const result = await newGameBoard(leaderboardId, newGameAt.toString());

			if (result) {
				result.subscribe(($result: any) => {
					if ($result.error) {
						console.error('Board creation error:', $result.error);
						alert('Failed to create board. Please try again.');
						boardCreationStartTime = null;
					} else if ($result.data) {
						// Wait 3 seconds before starting to poll
						setTimeout(() => {
							isNewGameCreated = true;
						}, 3000);
					}
				});
			}
		} catch (error) {
			console.error('Board creation failed:', error);
			alert('Failed to create board. Please make sure you are logged in.');
			boardCreationStartTime = null;
			isNewGameCreated = false;
		}
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
			// Only poll when actively waiting for new board
			if (isNewGameCreated) {
				board?.reexecute({ requestPolicy: 'network-only' });
				boards?.reexecute({ requestPolicy: 'network-only' });

				if (
					latestBoard?.boardId &&
					latestBoard.boardId !== boardId &&
					newGameAt &&
					latestBoard.createdAt &&
					Math.abs(parseInt(latestBoard.createdAt) - newGameAt) < 10000 &&
					latestBoard.leaderboardId === leaderboardId
				) {
					newGameAt = Date.now();
					isNewGameCreated = false;
					boardCreationStartTime = null;
					const url = new URL('/game', window.location.origin);
					url.searchParams.set('boardId', latestBoard.boardId);
					url.searchParams.set('leaderboardId', leaderboardId);

					setBoardId(latestBoard.boardId, leaderboardId);
					goto(url.toString(), { replaceState: false });
				}
			}
		}, 1000);

		return () => clearInterval(interval);
	});

	// Auto-create removed - user must manually click "New Game" button

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
				disabled={isCreatingBoard}
				class="text-md rounded-md border-none px-2 py-2 text-center font-bold text-[#f9f6f2] md:px-4 md:text-xl
				{canStartNewGame ? 'visible' : 'invisible'}
				{isCreatingBoard
					? 'cursor-not-allowed bg-[#9f8a76] opacity-70'
					: 'bg-[#8f7a66] hover:bg-[#9f8a76]'}"
			>
				{#if isCreatingBoard}
					<span class="animate-pulse">Creating...</span>
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
