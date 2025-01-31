<script lang="ts">
	import { queryStore, gql } from '@urql/svelte';

	import BoardHeader from '../molecules/BoardHeader.svelte';
	import { makeMoves } from '$lib/graphql/mutations/makeMove';
	import { onDestroy, onMount, createEventDispatcher } from 'svelte';
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
	import {
		moveHistoryStore,
		addMoveToHistory,
		flushMoveHistory,
		getMoveBatchForSubmission
	} from '$lib/stores/moveHistories';

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

	// State Management
	$: client = getClient(chainId ?? $userStore.chainId);
	let state: GameState | undefined;
	let isInitialized = false;
	let rendered = false;
	let blockHeight = 0;
	let isSynced: boolean = false;
	let stateHash = '';

	// Add new sync status tracking
	let syncStatus: 'idle' | 'syncing' | 'synced' | 'failed' = 'idle';
	let lastSyncTime: number | null = null;
	let pendingMoveCount = 0;
	let isFrozen = false;
	let consecutiveMismatches = 0; // Track consecutive mismatches
	let roundFirstMoveTime: number | null = null;

	// Add new state variable
	let offlineMode = false;
	const offlineMovesKey = (boardId: string) => `offlineMoves-${boardId}`;

	// GraphQL Queries and Subscriptions
	$: game = queryStore({
		client,
		query: GET_BOARD_STATE,
		variables: { boardId },
		requestPolicy: 'network-only'
	});

	// Reactive Statements
	$: score = $game.data?.board?.score || 0;

	$: if (isMultiplayer && $game.data?.board === null) {
		goto('/error');
	}

	$: if (!$game.fetching && $game.data?.board) {
		rendered = true;
	}

	$: boardEnded = isEnded || $game.data?.board?.isEnded || state?.finished;

	let isSetFinalScore = false;
	const updateScore = () => {
		if (!boardId || !$game.data?.board?.chainId) return;
		if (score <= bestScore) return;
		if ($game.data?.board?.player !== $userStore.username) return;
		const chainId = $game.data?.board?.chainId;
		const client = getClient(chainId);
		makeMoves(client, '[]', boardId);
	};

	$: if (!isSetFinalScore && boardId && boardEnded) {
		isSetFinalScore = true;
		updateScore();
	}

	$: if (boardId) {
		setGameCreationStatus(true);
	}

	$: bh = $game.data?.board?.reason?.NewBlock?.height;
	$: if (bh && bh !== blockHeight) {
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
		$game.data?.board &&
		boardId &&
		player &&
		(!isInitialized || $isNewGameCreated || $game.data?.board?.isEnded)
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

	const handleGameStateUpdate = () => {
		if (!boardId) return;
		state = createState($game.data?.board?.board, 4, boardId, player);
		isInitialized = true;

		if (state?.finished) {
			dispatch('end', {
				score: state.score,
				bestScore: Math.max(state.score, bestScore)
			});
		}

		isSynced = true;
		setGameCreationStatus(false);
	};

	// Movement Functions
	const move = async (boardId: string, direction: GameKeys) => {
		if (!canMakeMove || boardEnded || !state) return;

		const timestamp = Date.now().toString();

		// Keep local state management
		const prevTablet = boardToString(state?.tablet);
		state = await state?.actions[direction](state, timestamp, prevTablet);
		const newTablet = boardToString(state?.tablet);

		if (prevTablet === newTablet) return;
		if (!roundFirstMoveTime) {
			roundFirstMoveTime = Date.now();
		}

		// Add move to local history instead of immediate submission
		syncStatus = 'idle';
		pendingMoveCount++;
		addMoveToHistory({
			direction,
			timestamp,
			boardId
		});

		// Dispatch game over event if state changed to finished
		if (state?.finished) {
			dispatch('end', { score, bestScore });
		}
	};

	const handleMove = (direction: GameKeys, timestamp: string) => {
		if (!boardId) return;
		move(boardId, direction);
		dispatch('move', { direction, timestamp });
	};

	let idleTimeout: NodeJS.Timeout;
	let activityDetected = false;

	const setupIdleListener = () => {
		const events = ['mousemove', 'keydown', 'touchstart', 'click'];

		const resetTimer = () => {
			activityDetected = true;
			clearTimeout(idleTimeout);
			idleTimeout = setTimeout(() => handleIdleSubmit(), 2000);
		};

		events.forEach((e) => window.addEventListener(e, resetTimer));
		return () => events.forEach((e) => window.removeEventListener(e, resetTimer));
	};

	const handleIdleSubmit = async () => {
		const timeSinceFirstMove = roundFirstMoveTime ? Date.now() - roundFirstMoveTime : 0;

		// Cubic curve thresholds:
		// - 1 move: 5000ms (5s)
		// - 10 moves: 3200ms (3.2s)
		// - 20 moves: 400ms
		// - 25+ moves: 0ms
		const baseThreshold = 5000;
		const cubicFactor = 0.2; // Controls curve steepness
		const dynamicThreshold = Math.max(
			0,
			baseThreshold -
				pendingMoveCount * 200 - // Base linear reduction
				Math.max(0, pendingMoveCount - 10) ** 3 * cubicFactor // Cubic acceleration after 10 moves
		);

		// Force submit after 2s for 3+ moves
		const MIN_MOVE_FORCE_SUBMIT = 3;
		const forceSubmit = pendingMoveCount >= MIN_MOVE_FORCE_SUBMIT && timeSinceFirstMove >= 2000;

		// Early returns for invalid states
		if (!boardId) return;
		if (offlineMode) return;
		if (!activityDetected) return;
		if (pendingMoveCount === 0) return;

		// Handle game end case
		if (isEnded) {
			return submitMoves(boardId);
		}

		// Check if we should submit based on timing thresholds
		const shouldSubmit = timeSinceFirstMove >= dynamicThreshold || forceSubmit;
		if (!shouldSubmit) return;

		submitMoves(boardId);
	};

	const submitMoves = (boardId: string) => {
		syncStatus = 'syncing';
		const moves = flushMoveHistory(boardId);
		try {
			if (moves.length > 0) {
				makeMoves(client, getMoveBatchForSubmission(moves), boardId);
				const newTablet = boardToString(state?.tablet);
				stateHash = newTablet ?? '';
				isFrozen = true;
				syncStatus = 'syncing';
				pendingMoveCount = 0;
			}
		} catch (error) {
			syncStatus = 'failed';
			moveHistoryStore.update((history) => {
				const boardMoves = history.get(boardId as string) || [];
				return history.set(boardId as string, [...moves, ...boardMoves]);
			});
		} finally {
			activityDetected = false;
		}
	};

	// Add toggle handler
	const toggleOfflineMode = () => {
		offlineMode = !offlineMode;
		syncStatus = offlineMode ? 'idle' : 'syncing';
		localStorage.setItem('offlineModePreference', String(offlineMode));

		if (!offlineMode && boardId) {
			// Submit any stored moves when coming online
			submitMoves(boardId);
		}
	};

	// Lifecycle Hooks
	let initGameIntervalId: NodeJS.Timeout;
	onMount(() => {
		localBoardId = getBoardId(leaderboardId);
		if (!isMultiplayer && localBoardId && boardId === undefined) {
			boardId = localBoardId;
		}

		const cleanupListeners = setupIdleListener();

		// Try to get the board state
		game.reexecute({ requestPolicy: 'network-only' });
		initGameIntervalId = setInterval(() => {
			if (boardId && !$game.data?.board) {
				game.reexecute({ requestPolicy: 'network-only' });
			} else if ($game.data?.board) {
				clearInterval(initGameIntervalId);
			}
		}, 500);

		// Initialize from localStorage
		offlineMode = localStorage.getItem('offlineModePreference') === 'true';

		return () => {
			cleanupListeners();
			clearInterval(initGameIntervalId);
			clearTimeout(idleTimeout);
			// Submit any remaining moves when unmounting
			if (boardId) {
				const moves = flushMoveHistory(boardId);
				if (moves.length > 0) {
					makeMoves(client, getMoveBatchForSubmission(moves), boardId);
				}
			}
		};
	});

	let syncIntervalId: NodeJS.Timeout;
	onMount(() => {
		syncIntervalId = setInterval(() => {
			if (offlineMode) return; // Skip sync checks in offline mode
			if (boardId && (pendingMoveCount === 0 || syncStatus === 'syncing')) {
				// // Force check threshold conditions periodically
				// const timeSince = roundFirstMoveTime ? Date.now() - roundFirstMoveTime : 0;
				// const threshold = 5000 - Math.min(pendingMoveCount - 1, 49) * (5000 / 49);
				// console.log('timeSince', timeSince);
				// console.log('threshold', threshold);
				// if (timeSince >= threshold) {
				// 	handleIdleSubmit();
				// }

				game.reexecute({ requestPolicy: 'network-only' });
				if ($game.data?.board) {
					const backendBoardStr = boardToString($game.data.board.board);
					const localBoardStr = boardToString(state?.tablet);

					// State comparison logic with retry mechanism
					if (backendBoardStr !== localBoardStr) {
						consecutiveMismatches++;

						if (consecutiveMismatches >= 5) {
							// Confirm persistent mismatch, reset local state
							state = createState($game.data.board.board, 4, boardId, player);
							isFrozen = false;
							syncStatus = 'synced';
							roundFirstMoveTime = null;
							lastSyncTime = Date.now();
							consecutiveMismatches = 0; // Reset counter after resolution
						}
					} else {
						// States match, reset mismatch counter
						consecutiveMismatches = 0;

						if (syncStatus === 'syncing' && backendBoardStr === stateHash) {
							lastSyncTime = Date.now();
							isFrozen = false;
							roundFirstMoveTime = null;
							syncStatus = 'synced';
						}
					}
				}
			}
		}, 1000); // Check every second, 3 attempts = 3 seconds verification

		return () => {
			clearInterval(syncIntervalId);
		};
	});

	onDestroy(() => {
		setGameCreationStatus(false);
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
			canMakeMove={canMakeMove &&
				!boardEnded &&
				$game.data?.board?.player === $userStore.username &&
				!isFrozen}
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
	<div class="mt-2 flex items-center justify-center gap-4 text-xs lg:text-sm">
		<div
			class="bg-surface-800/50 border-surface-600/50 flex items-center gap-3 rounded-lg border px-4 py-2"
		>
			<div class="flex items-center gap-2">
				<span class="text-surface-400">Sync:</span>
				<div class="flex items-center gap-1.5">
					<div
						class="h-2 w-2 rounded-full
						{syncStatus === 'synced'
							? 'animate-pulse bg-emerald-500'
							: syncStatus === 'failed'
								? 'bg-red-500'
								: syncStatus === 'syncing'
									? 'animate-pulse bg-yellow-500'
									: 'bg-surface-400'}"
					></div>
					<span
						class="text-sm capitalize
						{syncStatus === 'synced'
							? 'text-emerald-400'
							: syncStatus === 'failed'
								? 'text-red-400'
								: syncStatus === 'syncing'
									? 'text-yellow-400'
									: 'text-surface-400'}"
					>
						{offlineMode ? 'Offline' : syncStatus}
					</span>
				</div>
			</div>

			<div class="bg-surface-600 h-4 w-px"></div>

			<div class="flex items-center gap-2">
				<span class="text-surface-400">Pending:</span>
				<span class="font-mono text-orange-400">{pendingMoveCount}</span>
			</div>

			{#if lastSyncTime}
				<div class="bg-surface-600 h-4 w-px"></div>
				<div class="flex items-center gap-2">
					<span class="text-surface-400">Last sync:</span>
					<span class="font-mono text-purple-400">
						{new Date(lastSyncTime).toLocaleTimeString([], {
							hour: '2-digit',
							minute: '2-digit',
							second: '2-digit'
						})}
					</span>
				</div>
			{/if}
		</div>

		<button
			on:click={toggleOfflineMode}
			class="bg-surface-800/50 border-surface-600/50 hover:bg-surface-700/50 flex items-center gap-2 rounded-lg border px-4 py-2 transition-colors"
		>
			<div class="h-2 w-2 rounded-full {offlineMode ? 'bg-orange-500' : 'bg-emerald-500'}"></div>
			<span class="text-surface-400 text-xs lg:text-sm">
				{offlineMode ? 'Go Online' : 'Go Offline'}
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
