<script lang="ts">
	import { queryStore, subscriptionStore, gql } from '@urql/svelte';

	import BoardHeader from '../molecules/BoardHeader.svelte';
	import { makeMove } from '$lib/graphql/mutations/makeMove';
	import { onDestroy, onMount, createEventDispatcher } from 'svelte';
	import { hashesStore, isHashesListVisible } from '$lib/stores/hashesStore';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { genInitialState as createState } from '$lib/game/game';
	import type { GameKeys, GameState } from '$lib/game/models';
	import { boardSize, isNewGameCreated, setGameCreationStatus } from '$lib/stores/gameStore';
	import { boardToString } from '$lib/game/utils';
	import Board from './Board.svelte';
	import { userStore } from '$lib/stores/userStore';
	import { getBoardId } from '$lib/stores/boardId';
	import { getClient } from '$lib/client';

	// Props
	export let isMultiplayer: boolean = false;
	export let isEnded: boolean = false;
	export let player: string;
	export let score: number = 0;
	export let bestScore: number = 0;
	export let boardId: string | undefined = undefined;
	export let chainId: string | undefined = undefined;
	export let canStartNewGame: boolean = true;
	export let canMakeMove: boolean = true;
	export let showBestScore: boolean = true;
	export let leaderboardId: string | undefined = undefined;
	export let overlayMessage: string | undefined = undefined;

	const dispatch = createEventDispatcher();

	// Board ID Management
	let localBoardId: string | null = null;

	// GraphQL Definitions
	const GET_BOARD_STATE = gql`
		query BoardState($boardId: Int!) {
			board(boardId: $boardId) {
				boardId
				board
				score
				isEnded
				player
				leaderboardId
				chainId
			}
		}
	`;

	const PLAYER_PING_SUBSCRIPTION = gql`
		subscription Notifications($chainId: ID!) {
			notifications(chainId: $chainId)
		}
	`;

	// State Management
	$: client = getClient(chainId ?? $userStore.chainId);
	let state: GameState | undefined;
	let isInitialized = false;
	let rendered = false;
	let blockHeight = 0;
	let lastHash = '';
	let moveStartTimes: Record<string, number> = {};
	let isSynced: boolean = false;

	// Timers and Flags
	let moveTimeout: NodeJS.Timeout | null = null;
	let syncTimeout: NodeJS.Timeout | null = null;
	let pingTime: number | null = null;
	let moveLimitMs = 350;

	let pingHistory: number[] = [];
	const MAX_PING_HISTORY = 5;

	const getMoveLimitMs = () => {
		if (!pingTime) {
			return 5000;
		}

		// Update ping history
		pingHistory = [...pingHistory, pingTime].slice(-MAX_PING_HISTORY);

		// Calculate average ping from recent history
		const avgPing = pingHistory.reduce((sum, p) => sum + p, 0) / pingHistory.length;

		const limits = [
			{ maxPing: 200, limitMs: 200 },
			{ maxPing: 500, limitMs: 400 },
			{ maxPing: 800, limitMs: 800 },
			{ maxPing: 1200, limitMs: 1200 },
			{ maxPing: 1800, limitMs: 1800 },
			{ maxPing: 2500, limitMs: 2500 }
		];

		// Add 20% buffer to the limit for stability
		const bufferMultiplier = 1.2;

		for (const { maxPing, limitMs } of limits) {
			if (avgPing < maxPing) {
				return Math.round(limitMs * bufferMultiplier);
			}
		}

		return 5000;
	};

	$: shouldSyncGame = false;

	// GraphQL Queries and Subscriptions
	$: game = queryStore({
		client,
		query: GET_BOARD_STATE,
		variables: { boardId },
		requestPolicy: 'network-only'
	});

	$: playerMessages = subscriptionStore({
		client,
		query: PLAYER_PING_SUBSCRIPTION,
		variables: { chainId }
	});

	// Reactive Statements
	$: score = $game.data?.board?.score || 0;

	$: if (isMultiplayer && $game.data?.board === null) {
		goto('/error');
	}

	$: if (!$game.fetching && $game.data?.board) {
		rendered = true;
	}

	$: boardEnded = isEnded || $game.data?.board?.isEnded;

	let isSetFinalScore = false;
	const updateScore = () => {
		if (!boardId || !$game.data?.board?.chainId) return;
		if (score <= bestScore) return;
		if ($game.data?.board?.player !== $userStore.username) return;
		const chainId = $game.data?.board?.chainId;
		const client = getClient(chainId);
		makeMove(client, '44', boardId);
	};

	$: if (!isSetFinalScore && boardId && boardEnded) {
		isSetFinalScore = true;
		updateScore();
	}

	$: if (boardId) {
		setGameCreationStatus(true);
	}

	$: bh = $playerMessages?.data?.notifications?.reason?.NewBlock?.height;
	$: if (bh && bh !== blockHeight) {
		handleNewBlock(bh);
		shouldRefetch = true;
	}

	let shouldRefetch = false;
	$: if (shouldRefetch) {
		setTimeout(() => {
			if ($userStore.username !== $game.data?.board?.player) {
				shouldRefetch = false;
				handleGameStateUpdate();
			}
		}, 1000);
	}

	$: if (
		$playerMessages?.data?.notifications?.reason?.NewBlock?.hash &&
		lastHash !== $playerMessages?.data?.notifications?.reason?.NewBlock?.hash
	) {
		handleNewHash($playerMessages?.data?.notifications?.reason?.NewBlock?.hash);
	}

	$: if (
		$game.data?.board &&
		boardId &&
		player &&
		(!isInitialized || $isNewGameCreated || $game.data?.board?.isEnded || shouldSyncGame)
	) {
		handleGameStateUpdate();
	}

	let isLeaderboardIdSet = false;
	$: {
		const gameLeaderboardId = $game.data?.board?.leaderboardId;
		if (!isLeaderboardIdSet) {
			if (gameLeaderboardId) {
				leaderboardId = gameLeaderboardId;
				isLeaderboardIdSet = true;
				const url = new URL($page.url);
				url.searchParams.set('leaderboardId', gameLeaderboardId);
				goto(url.toString(), { replaceState: true });
			} else if (gameLeaderboardId !== undefined) {
				leaderboardId = '';
			}
		}
	}

	// Utility Functions
	const hasWon = (board?: number[][]) => board?.some((row) => row?.some((cell) => cell >= 11));

	const getOverlayMessage = (board?: number[][]) => {
		if (overlayMessage) return overlayMessage;
		if (!isMultiplayer) {
			return hasWon(board) ? 'Congratulations! You Won!' : 'Game Over!';
		}
		return 'Game Over!';
	};

	// Game State Handlers
	const handleNewBlock = (newBlockHeight: number) => {
		blockHeight = newBlockHeight;
		canMakeMove = true;
		if (moveTimeout) clearTimeout(moveTimeout);

		const lastMove = Object.entries(moveStartTimes)[0];
		if (lastMove) {
			const [direction, startTime] = lastMove;
			pingTime = Date.now() - startTime;
			delete moveStartTimes[direction];
		}
		game.reexecute({ requestPolicy: 'network-only' });
	};

	const handleNewHash = (hash: string) => {
		lastHash = hash;
		if (lastHash) {
			hashesStore.update((logs) => [
				{ hash: lastHash, timestamp: new Date().toISOString() },
				...logs
			]);
		}
	};

	const handleGameStateUpdate = () => {
		if (!boardId) return;
		state = createState($game.data?.board?.board, 4, boardId, player);
		isInitialized = true;
		shouldSyncGame = false;
		isSynced = true;
		setGameCreationStatus(false);
	};

	// Movement Functions
	const move = async (boardId: string, direction: GameKeys) => {
		if (!canMakeMove || $game.data?.board?.isEnded) return;

		canMakeMove = false;
		shouldSyncGame = false;
		isSynced = false;
		moveStartTimes[direction] = Date.now();
		moveLimitMs = getMoveLimitMs();

		moveTimeout = setTimeout(() => {
			canMakeMove = true;
		}, moveLimitMs);

		if (syncTimeout) clearTimeout(syncTimeout);
		syncTimeout = setTimeout(() => {
			shouldSyncGame = true;
		}, 2000);

		const timestamp = Date.now().toString();
		makeMove(client, timestamp, boardId, direction);

		const prevTablet = boardToString(state?.tablet);
		state = await state?.actions[direction](state, timestamp, prevTablet);
	};

	const handleMove = (direction: GameKeys, timestamp: string) => {
		if (!boardId) return;
		move(boardId, direction);
		dispatch('move', { direction, timestamp });
	};

	// Lifecycle Hooks
	let intervalId: NodeJS.Timeout;
	onMount(() => {
		localBoardId = getBoardId(leaderboardId);
		if (!isMultiplayer && localBoardId && boardId === undefined) {
			boardId = localBoardId;
		}

		game.reexecute({ requestPolicy: 'network-only' });
		intervalId = setInterval(() => {
			if (boardId && !$game.data?.board) {
				game.reexecute({ requestPolicy: 'network-only' });
			} else if ($game.data?.board) {
				clearInterval(intervalId);
			}
		}, 500);

		return () => clearInterval(intervalId);
	});

	onDestroy(() => {
		if (playerMessages) {
			playerMessages.pause();
			hashesStore.set([]);
			setGameCreationStatus(false);
		}
	});

	$: overlayMessage =
		$game.data?.board?.player === $userStore.username
			? getOverlayMessage($game.data?.board?.board)
			: $game.data?.board?.player;
</script>

<div class="game-container {$boardSize}">
	<div class="game-board">
		<Board
			tablet={state?.tablet}
			canMakeMove={canMakeMove && $game.data?.board?.player === $userStore.username}
			isEnded={boardEnded}
			{overlayMessage}
			moveCallback={handleMove}
		>
			<!-- {#snippet header(size)} -->
			{#snippet header()}
				<BoardHeader
					bind:boardId
					{canStartNewGame}
					{showBestScore}
					player={$game.data?.board?.player ?? $userStore.username}
					{score}
					{bestScore}
				/>
			{/snippet}
		</Board>
	</div>
	<div class="mt-2 flex items-center justify-center gap-4 text-sm">
		<button
			class="bg-surface-800/50 flex items-center gap-2 rounded-lg px-3 py-1.5 transition-colors hover:bg-black/50"
			on:click={() => isHashesListVisible.update((current) => !current)}
		>
			<div
				class="h-2 w-2 rounded-full transition-colors duration-300 {!canMakeMove
					? 'bg-red-500'
					: 'bg-emerald-500'}"
				title={`Move limit: ${moveLimitMs}ms`}
			></div>
			<span
				class="cursor-pointer font-mono text-emerald-400"
				title={lastHash || 'No hash available'}
			>
				{#if lastHash}
					{lastHash.slice(0, 6)}...{lastHash.slice(-4)}
				{:else}
					---
				{/if}
			</span>
			<span class="text-surface-400">|</span>
			<span class="text-orange-400"
				>{pingTime || 0}<span class="text-surface-400 ml-1 text-xs">ms</span></span
			>
			<span class="text-surface-400">|</span>
			<span class={isSynced ? 'text-emerald-400' : 'text-yellow-400'}>
				{isSynced ? 'synced' : 'syncing'}
			</span>
		</button>
	</div>
</div>

<style>
	.game-container {
		margin: 0 auto;
		text-align: center;
		overflow: visible;
	}

	.game-container.lg {
		max-width: 555px;
	}

	.game-container.md {
		max-width: 460px;
	}

	.game-container.sm {
		max-width: 370px;
	}

	.game-board {
		position: relative;
		width: 100%;
	}
</style>
