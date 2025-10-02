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
	import { deleteBoardId, getBoardId } from '$lib/stores/boardId';
	import { getClient } from '$lib/client';
	import {
		moveHistoryStore,
		addMoveToHistory,
		flushMoveHistory,
		getMoveBatchForSubmission
	} from '$lib/stores/moveHistories';
	import { formatBalance } from '$lib/utils/formatBalance';
	import { requestFaucetMutation } from '$lib/graphql/mutations/requestFaucet';

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
	let isCreatingNewBoard: boolean = false;

	// GraphQL Definitions
	const GET_BOARD_STATE = gql`
		query BoardState($boardId: String!) {
			board(boardId: $boardId) {
				boardId
				board
				score
				isEnded
				player
				leaderboardId
				chainId
				shardId
				createdAt
				endTime
				totalMoves
				moveHistory {
					direction
					timestamp
					boardAfter
					scoreAfter
				}
			}
			balance
		}
	`;

	// State Management
	$: client = getClient(chainId ?? $userStore.chainId);
	let state: GameState | undefined;
	let isInitialized = false;
	let rendered = false;
	let isSynced: boolean = false;
	let stateHash = '';

	// Add new sync status tracking
	let syncStatus: 'idle' | 'syncing' | 'synced' | 'failed' = 'idle';
	let lastSyncTime: number | null = null;
	let pendingMoveCount = 0;
	let isFrozen = false;
	let consecutiveMismatches = 0; // Track consecutive mismatches
	let roundFirstMoveTime: number | null = null;

	// Offline mode disabled for website
	const offlineMode = false;
	const toggleOfflineMode = () => {};

	// Add new move processing flag
	let isProcessingMove = false;

	// Add new balance view state
	let showBalance = false;

	// 🔍 Inspector Mode: viewing someone else's board
	let isInspectorMode = false;
	let inspectorMoveHistory: any[] = [];
	let inspectorCurrentMoveIndex = 0;
	let isInspectorPlaying = false;
	let inspectorPlayTimeout: NodeJS.Timeout | null = null;
	let autoPlayEnabled = false; // Toggle state for auto-play
	let previousMoveHistoryLength = 0; // Track previous length to detect new moves

	// GraphQL Queries and Subscriptions
	$: game = queryStore({
		client,
		query: GET_BOARD_STATE,
		variables: { boardId },
		requestPolicy: 'network-only'
	});

	// Reactive Statements
	$: score = $game.data?.board?.score || 0;

	// 🔍 Check if inspector mode (viewing someone else's board)
	$: {
		const boardPlayer = $game.data?.board?.player;
		const currentUser = $userStore.username;
		if (boardPlayer && currentUser && boardPlayer !== currentUser) {
			isInspectorMode = true;
			const newMoveHistory = $game.data?.board?.moveHistory || [];
			const newMovesAdded = newMoveHistory.length > previousMoveHistoryLength && previousMoveHistoryLength > 0;
			const wasAtEnd = inspectorCurrentMoveIndex === previousMoveHistoryLength;
			
			// Store old length before updating
			const oldLength = inspectorMoveHistory.length;
			inspectorMoveHistory = newMoveHistory;
			
			// Auto-advance to latest move on first load
			if (inspectorCurrentMoveIndex === 0 && inspectorMoveHistory.length > 0) {
				inspectorCurrentMoveIndex = inspectorMoveHistory.length;
				previousMoveHistoryLength = inspectorMoveHistory.length;
			}
			// If auto-play is enabled and new moves were added while we were at the end
			else if (autoPlayEnabled && newMovesAdded && wasAtEnd && !isInspectorPlaying) {
				// Don't update inspectorCurrentMoveIndex here - let playInspectorMoves handle it
				previousMoveHistoryLength = newMoveHistory.length;
				playInspectorMoves();
			} else {
				previousMoveHistoryLength = newMoveHistory.length;
			}
		} else {
			isInspectorMode = false;
		}
	}

	$: if (isMultiplayer && $game.data?.board === null) {
		goto('/error');
	}

	$: if (!$game.fetching && $game.data?.board) {
		rendered = true;
	}

	$: boardEnded = isEnded || $game.data?.board?.isEnded || state?.finished;

	$: if (boardEnded) {
		deleteBoardId(leaderboardId);
		if (offlineMode) {
			toggleOfflineMode();
		}
		setTimeout(() => {
			const isLeaderboardEnded = parseInt($game.data?.board?.endTime) <= Date.now();
			if (!isSetFinalScore && isLeaderboardEnded) {
				isSetFinalScore = true;
				updateLeaderboardScore();
			}
		}, 2000);
	}

	let isSetFinalScore = false;
	const updateLeaderboardScore = () => {
		if (!boardId || !$game.data?.board?.chainId) return;
		if (score <= bestScore) return;
		if ($game.data?.board?.player !== $userStore.username) return;
		const chainId = $game.data?.board?.chainId;
		const client = getClient(chainId);
		makeMoves(client, '[]', boardId);
	};

	$: if (boardId) {
		setGameCreationStatus(true);
	}

	// Block height tracking removed - reason field no longer exists in new schema

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
		
		// 🔍 In inspector mode, use move history to show board state
		if (isInspectorMode && inspectorMoveHistory.length > 0) {
			// inspectorCurrentMoveIndex: 1 = after move 1, 2 = after move 2, etc.
			const moveIndex = inspectorCurrentMoveIndex;
			if (moveIndex >= 1 && moveIndex <= inspectorMoveHistory.length) {
				// Show board state after the selected move
				const moveData = inspectorMoveHistory[moveIndex - 1];
				state = createState(moveData.boardAfter, 4, boardId, player);
			} else {
				// Show current/final board state
				state = createState($game.data?.board?.board, 4, boardId, player);
			}
		} else {
			// Normal mode: use current board state
			state = createState($game.data?.board?.board, 4, boardId, player);
		}
		
		isInitialized = true;

		if (state?.finished && !isInspectorMode) {
			dispatch('end', {
				score: state.score,
				bestScore: Math.max(state.score, bestScore)
			});
			// Force submit moves when game ends
			submitMoves(boardId, true);
		}

		isSynced = true;
		setGameCreationStatus(false);
	};

	// Movement Functions
	const move = async (boardId: string, direction: GameKeys) => {
		if (!canMakeMove || boardEnded || !state || isProcessingMove) return;
		isProcessingMove = true;

		try {
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
		} finally {
			isProcessingMove = false;
		}
	};

	let lastMoveTime = 0;
	const MOVE_COOLDOWN = 50; // 50ms minimum between moves

	const handleMove = (direction: GameKeys, timestamp: string) => {
		const now = Date.now();
		if (now - lastMoveTime < MOVE_COOLDOWN) return;
		lastMoveTime = now;

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
		if (!boardId) return;

		// Handle game end case
		if (boardEnded && pendingMoveCount > 0) {
			return submitMoves(boardId, true);
		}

		// Early returns for invalid states
		if (offlineMode) return;
		if (!activityDetected) return;
		if (pendingMoveCount === 0) return;

		// Check if we should submit based on timing thresholds
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
		const shouldSubmit = timeSinceFirstMove >= dynamicThreshold || forceSubmit;
		if (!shouldSubmit) return;

		submitMoves(boardId);
	};

	const submitMoves = (boardId: string, force = false) => {
		// Offline mode disabled for website - always submit moves
		const moves = flushMoveHistory(boardId);
		try {
			if ((moves.length > 0 || force)) {
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

	// Offline mode toggle removed for website

	// Add toggle handler for balance view
	let dirtyBalance = false;
	const toggleBalanceView = () => {
		dirtyBalance = true;
		showBalance = !showBalance;
	};

	const requestFaucet = () => {
		requestFaucetMutation(client);
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

		// Offline mode disabled for website

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

	// 🔍 Inspector Auto-Play Functions
	const toggleAutoPlay = () => {
		autoPlayEnabled = !autoPlayEnabled;
		
		if (autoPlayEnabled) {
			// Start playing if not already playing
			if (!isInspectorPlaying) {
				playInspectorMoves();
			}
		} else {
			// Stop playing when disabled
			stopInspectorPlay();
		}
	};

	const playInspectorMoves = () => {
		if (inspectorCurrentMoveIndex >= inspectorMoveHistory.length) {
			inspectorCurrentMoveIndex = 0;
		}
		isInspectorPlaying = true;
		playNextInspectorMove();
	};

	const playNextInspectorMove = () => {
		if (!isInspectorPlaying || inspectorCurrentMoveIndex >= inspectorMoveHistory.length) {
			stopInspectorPlay();
			return;
		}

		const currentMove = inspectorMoveHistory[inspectorCurrentMoveIndex];
		const nextMove = inspectorMoveHistory[inspectorCurrentMoveIndex + 1];

		// Calculate delay based on timestamp difference
		let delay = 500; // Default 500ms
		if (nextMove) {
			const currentTime = parseInt(currentMove.timestamp);
			const nextTime = parseInt(nextMove.timestamp);
			delay = Math.min(nextTime - currentTime, 2000); // Cap at 2 seconds
		}

		inspectorCurrentMoveIndex++;
		handleGameStateUpdate();

		inspectorPlayTimeout = setTimeout(() => {
			playNextInspectorMove();
		}, delay);
	};

	const stopInspectorPlay = () => {
		isInspectorPlaying = false;
		if (inspectorPlayTimeout) {
			clearTimeout(inspectorPlayTimeout);
			inspectorPlayTimeout = null;
		}
	};

	const restartInspector = () => {
		stopInspectorPlay();
		inspectorCurrentMoveIndex = 1; // Start at first move
		handleGameStateUpdate();
	};

	onDestroy(() => {
		setGameCreationStatus(false);
		stopInspectorPlay();
	});

	$: overlayMessage =
		$game.data?.board?.player === $userStore.username
			? getOverlayMessage($game.data?.board?.board)
			: $game.data?.board?.player;

	// $: if (!$game.fetching && parseFloat($game.data?.balance ?? '0.00') <= 0.2 && !dirtyBalance) {
	// 	showBalance = true;
	// 	dirtyBalance = false;
	// }
</script>

<div class="game-container {$boardSize}">
	<div class="game-board">
		<Board
			tablet={state?.tablet}
			canMakeMove={!isInspectorMode &&
				canMakeMove &&
				!boardEnded &&
				$game.data?.board?.player === $userStore.username &&
				!isFrozen &&
				!isCreatingNewBoard}
			isEnded={boardEnded}
			{overlayMessage}
			moveCallback={handleMove}
			{boardId}
			{chainId}
			showReplayButton={true}
		>
			<!-- {#snippet header(size)} -->
			{#snippet header()}
				<BoardHeader
					bind:boardId
					bind:isCreating={isCreatingNewBoard}
					{canStartNewGame}
					{showBestScore}
					player={$game.data?.board?.player ?? $userStore.username}
					{score}
					{bestScore}
				/>
			{/snippet}
		</Board>
	</div>
	{#if $userStore.username}
		<div
			class="mt-2 flex flex-col items-center justify-center gap-y-2 text-xs lg:flex-row lg:gap-3 lg:text-sm"
		>
			<div
				class="flex w-full cursor-pointer flex-wrap items-center gap-x-2 gap-y-2 rounded-lg border border-surface-600/50 bg-surface-800/50 px-4 py-2 transition-all lg:w-auto"
			>
				{#if showBalance}
					<!-- Balance View -->
					<div class="flex w-full items-center justify-between gap-3">
						<button class="flex w-full flex-grow items-center gap-2" onclick={toggleBalanceView}>
							<span class="text-surface-400">Balance:</span>
							<span class="font-mono text-emerald-400">{formatBalance($game.data?.balance)}</span>
						</button>

						<button
							onclick={requestFaucet}
							class="ms-8 rounded-sm font-bold text-white transition-colors enabled:hover:text-orange-400 disabled:opacity-50"
							disabled={parseFloat($game.data?.balance ?? '0.00') > 0.2}
						>
							Faucet
						</button>
					</div>
				{:else}
					<!-- Original Status View -->
					<button class="flex w-full items-center gap-3" onclick={toggleBalanceView}>
						<div class="flex w-full items-center gap-3">
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
										class="text-xs capitalize lg:text-sm
								{syncStatus === 'synced'
											? 'text-emerald-400'
											: syncStatus === 'failed'
												? 'text-red-400'
												: syncStatus === 'syncing'
													? 'text-yellow-400'
													: 'text-surface-400'}"
									>
										{syncStatus}
									</span>
								</div>
							</div>

							<div class="h-4 w-px bg-surface-600"></div>

							<div class="flex flex-grow items-center gap-2">
								<span class="text-surface-400">Pending:</span>
								<span class="font-mono text-orange-400">{pendingMoveCount}</span>
							</div>
						</div>

						{#if lastSyncTime}
							<div class="h-px w-full bg-surface-600 lg:h-4 lg:w-px"></div>
							<div class="flex w-full items-center gap-2 whitespace-nowrap lg:w-auto">
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
					</button>
				{/if}
			</div>

			<!-- Offline mode toggle hidden for website -->
			{#if false}
				<button
					onclick={toggleOfflineMode}
					class="flex w-full items-center gap-2 rounded-lg border border-surface-600/50 bg-surface-800/50 px-4 py-2 transition-colors hover:bg-surface-700/50 lg:w-auto"
				>
					<div class="h-2 w-2 rounded-full {offlineMode ? 'bg-orange-500' : 'bg-emerald-500'}"></div>
					<span class="whitespace-nowrap text-xs text-surface-400 lg:text-sm"
						>{offlineMode ? 'Go Online' : 'Go Offline'}</span
					>
				</button>
			{/if}
		</div>
	{/if}

	<!-- 🔍 Inspector Mode Controls -->
	{#if isInspectorMode && inspectorMoveHistory.length > 0}
		<div class="mt-4 rounded-lg border border-purple-500/50 bg-purple-900/20 p-4">
			<div class="mb-2 flex items-center justify-between">
				<div class="flex items-center gap-2">
					<div class="h-2 w-2 rounded-full bg-purple-500"></div>
					<span class="text-sm font-bold text-purple-400">Inspector Mode</span>
				</div>
				<div class="text-xs text-surface-400">
					Move {inspectorCurrentMoveIndex} / {inspectorMoveHistory.length}
				</div>
			</div>

			<!-- Progress Slider -->
			<input
				type="range"
				min="1"
				max={inspectorMoveHistory.length}
				bind:value={inspectorCurrentMoveIndex}
				oninput={() => {
					stopInspectorPlay();
					handleGameStateUpdate();
				}}
				class="inspector-slider w-full"
			/>

			<!-- Playback Controls -->
			<div class="mt-3 flex items-center justify-center gap-2">
				<button
					onclick={restartInspector}
					class="rounded-md bg-surface-700 px-3 py-1.5 text-xs font-bold text-white transition-colors hover:bg-surface-600"
				>
					⏮ Restart
				</button>

				<button
					onclick={toggleAutoPlay}
					class="rounded-md px-4 py-1.5 text-xs font-bold text-white transition-colors {autoPlayEnabled
						? 'bg-orange-500 hover:bg-orange-600'
						: 'bg-emerald-500 hover:bg-emerald-600'}"
				>
					{#if autoPlayEnabled}
						⏸ Auto-Play ON
					{:else}
						▶ Auto-Play OFF
					{/if}
				</button>

				<button
					onclick={() => {
						if (inspectorCurrentMoveIndex < inspectorMoveHistory.length) {
							inspectorCurrentMoveIndex++;
							handleGameStateUpdate();
						}
					}}
					disabled={inspectorCurrentMoveIndex >= inspectorMoveHistory.length}
					class="rounded-md bg-surface-700 px-3 py-1.5 text-xs font-bold text-white transition-colors hover:bg-surface-600 disabled:opacity-50"
				>
					Step →
				</button>
			</div>

			<div class="mt-2 flex items-center justify-center gap-2 text-xs text-surface-500">
				<span>Viewing {$game.data?.board?.player}'s game</span>
				{#if autoPlayEnabled}
					<span class="animate-pulse text-emerald-400">
						• Auto-Play {isInspectorPlaying ? 'Active' : 'Waiting for moves'}
					</span>
				{/if}
			</div>
		</div>
	{/if}
</div>

<style>
	.inspector-slider {
		-webkit-appearance: none;
		appearance: none;
		background: #3a3a3c;
		height: 6px;
		border-radius: 3px;
		outline: none;
	}

	.inspector-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 16px;
		height: 16px;
		background: #a855f7;
		cursor: pointer;
		border-radius: 50%;
	}

	.inspector-slider::-moz-range-thumb {
		width: 16px;
		height: 16px;
		background: #a855f7;
		cursor: pointer;
		border-radius: 50%;
		border: none;
	}

	.game-container {
		margin: 0 auto;
		text-align: center;
		overflow: visible;
		transition: all 0.2s ease-in-out;
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
