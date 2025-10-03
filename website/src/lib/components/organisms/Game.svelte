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
	let lastBoardId: string | undefined = undefined; // Track board changes

	// Add new sync status tracking
	let syncStatus:
		| 'ready'
		| 'pending'
		| 'syncing'
		| 'syncing-bg'
		| 'synced'
		| 'failed'
		| 'desynced' = 'ready';
	let lastSyncTime: number | null = null;
	let pendingMoveCount = 0;
	let isFrozen = false;
	let lastHashMismatchTime: number | null = null; // Track when hash mismatch started
	let roundFirstMoveTime: number | null = null;

	// 🔄 Background Sync: Valid board states tracking (hash-based validation)
	let validBoardHashes: Set<number> = new Set(); // All board states we've seen locally

	// 🔄 Background Sync: Activity Tracking (improved for mixed play styles)
	let recentMoves: number[] = [];
	const ACTIVITY_WINDOW = 10000; // 10 seconds - longer window for better pattern detection

	// 🔄 Background Sync: Config (optimized for batching to reduce backend load)
	const BURST_ACTIVITY_THRESHOLD = 3; // moves per second (short burst)
	const SUSTAINED_ACTIVITY_THRESHOLD = 1.5; // moves per second (over longer period)
	const BURST_SYNC_INTERVAL = 12000; // 12 seconds (8-15 range for bursts)
	const SUSTAINED_PENDING_LIMIT = 25; // 25 moves for steady players
	const LOW_ACTIVITY_PENDING_LIMIT = 15; // moves for slow players
	const VERY_LOW_ACTIVITY_THRESHOLD = 0.5; // moves per second (very slow play)

	// Hash function for quick board comparison
	const hashBoard = (board: any) => {
		let hash = 2166136261;
		for (const row of board) {
			for (const cell of row) {
				const value = typeof cell === 'number' ? cell : (cell?.value ?? 0);
				hash ^= value;
				hash = (hash * 16777619) >>> 0; // Keep as 32-bit unsigned
			}
		}
		return hash;
	};

	// Track move activity with better pattern detection
	const trackMoveActivity = () => {
		const now = Date.now();
		recentMoves.push(now);
		recentMoves = recentMoves.filter((t) => now - t < ACTIVITY_WINDOW);
	};

	// Calculate sustained activity over the full window
	const getSustainedActivity = () => {
		if (recentMoves.length === 0) return 0;
		const windowSeconds = ACTIVITY_WINDOW / 1000;
		return recentMoves.length / windowSeconds; // moves per second over full window
	};

	// Calculate recent burst activity (last 2 seconds)
	const getBurstActivity = () => {
		const now = Date.now();
		const recentBurst = recentMoves.filter((t) => now - t < 2000); // last 2 seconds
		return recentBurst.length / 2; // moves per second in recent burst
	};

	// Determine activity level based on patterns
	const getActivityLevel = () => {
		const sustained = getSustainedActivity();
		const burst = getBurstActivity();

		if (burst >= BURST_ACTIVITY_THRESHOLD) return 'burst'; // High burst activity
		if (sustained >= SUSTAINED_ACTIVITY_THRESHOLD) return 'sustained'; // Steady play
		if (sustained >= VERY_LOW_ACTIVITY_THRESHOLD) return 'moderate'; // Moderate activity
		return 'low'; // Very slow or paused
	};

	// Add board state to valid hashes set
	const addValidBoardHash = (tablet: any) => {
		if (!tablet) return;
		const hash = hashBoard(tablet);
		validBoardHashes.add(hash);
	};

	// Offline mode disabled for website
	const offlineMode = false;
	const toggleOfflineMode = () => {};

	// Add new move processing flag and queue
	let isProcessingMove = false;
	let moveQueue: Array<{ direction: GameKeys; timestamp: string }> = [];

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
	let hideInspectorOverlay = true; // Control inspector overlay visibility (start hidden)
	let lastInspectedBoardId: string | undefined = undefined; // Track board changes

	// GraphQL Queries and Subscriptions
	$: game = queryStore({
		client,
		query: GET_BOARD_STATE,
		variables: { boardId },
		requestPolicy: 'network-only'
	});

	// Reactive Statements
	$: score = $game.data?.board?.score || 0;

	// 🔍 Check if inspector mode (viewing someone else's board OR own ended game)
	$: {
		const boardPlayer = $game.data?.board?.player;
		const currentUser = $userStore.username;
		const gameEnded = $game.data?.board?.isEnded;
		const currentBoardId = $game.data?.board?.boardId;
		
		// Only check inspector mode if board data matches requested boardId (avoid stale data)
		const isBoardDataValid = currentBoardId === boardId;
		const isOtherPlayer = isBoardDataValid && boardPlayer && currentUser && boardPlayer !== currentUser;
		const isOwnEndedGame = isBoardDataValid && boardPlayer === currentUser && gameEnded;

		if (isOtherPlayer || isOwnEndedGame) {
			const wasInspectorMode = isInspectorMode;
			const isBoardChanged = currentBoardId && currentBoardId !== lastInspectedBoardId;

			isInspectorMode = true;
			const newMoveHistory = $game.data?.board?.moveHistory || [];
			const newMovesAdded =
				newMoveHistory.length > previousMoveHistoryLength && previousMoveHistoryLength > 0;
			const wasAtEnd = inspectorCurrentMoveIndex === previousMoveHistoryLength;

			// Update move history without triggering state update
			inspectorMoveHistory = newMoveHistory;

			// Reset to latest move when switching to a different board
			if (isBoardChanged) {
				lastInspectedBoardId = currentBoardId;
				inspectorCurrentMoveIndex = inspectorMoveHistory.length;
				previousMoveHistoryLength = inspectorMoveHistory.length;
				handleGameStateUpdate();

				// Show overlay if at end and game ended
				if (gameEnded && inspectorCurrentMoveIndex >= inspectorMoveHistory.length) {
					hideInspectorOverlay = false;
				} else {
					hideInspectorOverlay = true;
				}
			}
			// Auto-advance to latest move on first load
			else if (!wasInspectorMode && inspectorMoveHistory.length > 0) {
				lastInspectedBoardId = currentBoardId;
				inspectorCurrentMoveIndex = inspectorMoveHistory.length;
				previousMoveHistoryLength = inspectorMoveHistory.length;
				handleGameStateUpdate();

				// Show overlay if at end and game ended
				if (gameEnded && inspectorCurrentMoveIndex >= inspectorMoveHistory.length) {
					hideInspectorOverlay = false;
				} else {
					hideInspectorOverlay = true;
				}
			}
			// If auto-play is enabled and new moves were added while we were at the end
			else if (autoPlayEnabled && newMovesAdded && wasAtEnd && !isInspectorPlaying) {
				// Don't update inspectorCurrentMoveIndex here - let playInspectorMoves handle it
				previousMoveHistoryLength = newMoveHistory.length;
				playInspectorMoves();
			} else {
				// Just update the length, don't trigger state update
				previousMoveHistoryLength = newMoveHistory.length;
			}
		} else {
			isInspectorMode = false;
			lastInspectedBoardId = undefined;
		}
	}

	// 🔍 Reactive statement for inspector mode board state and score updates
	// Only updates when inspectorCurrentMoveIndex changes, not on every poll
	$: if (
		isInspectorMode &&
		inspectorMoveHistory.length > 0 &&
		inspectorCurrentMoveIndex > 0 &&
		boardId
	) {
		const moveIndex = inspectorCurrentMoveIndex;
		if (moveIndex >= 1 && moveIndex <= inspectorMoveHistory.length) {
			// Show board state and score after the selected move
			const moveData = inspectorMoveHistory[moveIndex - 1];
			state = createState(moveData.boardAfter, 4, boardId, player);
			score = moveData.scoreAfter;
		} else {
			// Show current/final board state and score
			state = createState($game.data?.board?.board, 4, boardId, player);
			score = $game.data?.board?.score || 0;
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
		moveQueue = [];

		deleteBoardId(leaderboardId);
		if (offlineMode) {
			toggleOfflineMode();
		}

		const endTime = parseInt($game.data?.board?.endTime);
		const isLeaderboardEnded = endTime > 0 && endTime <= Date.now();
		if (!isSetFinalScore && isLeaderboardEnded) {
			isSetFinalScore = true;
			updateLeaderboardScore();
		}
	}

	let isSetFinalScore = false;
	const updateLeaderboardScore = () => {
		if (!boardId || !$game.data?.board?.chainId) return;
		if ($game.data?.board?.player !== $userStore.username) return;

		if (pendingMoveCount > 0) {
			submitMoves(boardId, true);
		}

		const chainId = $game.data?.board?.chainId;
		const client = getClient(chainId);
		const finalBoardId = boardId;

		setTimeout(() => {
			if (finalBoardId) {
				makeMoves(client, '[]', finalBoardId);
			}
		}, 500);
	};

	$: if (boardId) {
		setGameCreationStatus(true);
	}

	// Block height tracking removed - reason field no longer exists in new schema

	$: if (
		$game.data?.board &&
		boardId &&
		player &&
		!isInspectorMode && // Don't auto-update in inspector mode
		(!isInitialized || // Initial load
			($game.data?.board?.isEnded && !boardEnded) || // Game just ended
			boardId !== lastBoardId) // New board created
	) {
		// Reset state for new board
		if (boardId !== lastBoardId) {
			isInitialized = false;
			validBoardHashes.clear();
			pendingMoveCount = 0;
			syncStatus = 'ready';
			lastHashMismatchTime = null;
			lastBoardId = boardId;
		}
		handleGameStateUpdate();
	}

	// Handle tournament ending while player was offline
	$: if ($game.data?.board && boardId && player && !isInspectorMode && isInitialized) {
		const tournamentEndTime = parseInt($game.data?.board?.endTime || '0');
		const boardIsActive = !$game.data?.board?.isEnded;
		const tournamentEnded = tournamentEndTime > 0 && tournamentEndTime <= Date.now();
		const isOwnBoard = $game.data?.board?.player === player;

		if (tournamentEnded && boardIsActive && !isSetFinalScore && isOwnBoard) {
			isSetFinalScore = true;
			updateLeaderboardScore();
		}
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

		// Normal mode: use current board state
		state = createState($game.data?.board?.board, 4, boardId, player);

		// Register initial board state as valid to prevent false desync detection
		addValidBoardHash(state?.tablet);

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

		const tournamentEndTime = parseInt($game.data?.board?.endTime || '0');
		if (tournamentEndTime > 0 && tournamentEndTime <= Date.now()) {
			return;
		}

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

			// 🔄 Track move activity for background sync
			trackMoveActivity();

			// 🔄 Add board state to valid hashes
			addValidBoardHash(state?.tablet);

			// Add move to local history instead of immediate submission
			// Don't reset syncStatus if currently syncing
			if (syncStatus !== 'syncing-bg' && syncStatus !== 'syncing') {
				syncStatus = 'pending';
			}
			pendingMoveCount++;
			addMoveToHistory({
				direction,
				timestamp,
				boardId
			});

			// Dispatch game over event if state changed to finished
			if (state?.finished) {
				dispatch('end', { score, bestScore });
				// Immediately sync when game ends to submit final score
				if (pendingMoveCount > 0) {
					submitMoves(boardId, true);
				}
			} else {
				// 🔄 Check if we should trigger background sync (only during active play)
				checkBackgroundSyncTriggers();
			}
		} finally {
			isProcessingMove = false;
			// Process queued moves
			processQueuedMoves(boardId);
		}
	};

	// Process queued moves sequentially
	const processQueuedMoves = async (boardId: string) => {
		if (moveQueue.length === 0 || isProcessingMove) return;

		const nextMove = moveQueue.shift();
		if (nextMove && !boardEnded) {
			await move(boardId, nextMove.direction);
		}
	};

	let lastMoveTime = 0;
	const MOVE_COOLDOWN = 50; // 50ms minimum between moves

	const handleMove = (direction: GameKeys, timestamp: string) => {
		const now = Date.now();
		if (now - lastMoveTime < MOVE_COOLDOWN) return;
		lastMoveTime = now;

		if (!boardId) return;

		const tournamentEndTime = parseInt($game.data?.board?.endTime || '0');
		if (tournamentEndTime > 0 && tournamentEndTime <= Date.now()) {
			return;
		}

		// Queue move if currently processing, otherwise execute immediately
		if (isProcessingMove) {
			moveQueue.push({ direction, timestamp });
		} else {
			move(boardId, direction);
		}

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

	// 🔄 Background Sync: Check if we should trigger sync (optimized for batching)
	const checkBackgroundSyncTriggers = () => {
		if (!boardId || pendingMoveCount === 0) return;

		const activityLevel = getActivityLevel();
		const timeSinceLastSync = lastSyncTime ? Date.now() - lastSyncTime : Infinity;

		// Condition 1: BURST Activity - Sync every 12 seconds (larger batches)
		if (activityLevel === 'burst' && timeSinceLastSync > BURST_SYNC_INTERVAL) {
			backgroundSync(boardId);
			return;
		}

		// Condition 2: SUSTAINED Activity - Sync every 25 moves (big batches)
		if (activityLevel === 'sustained' && pendingMoveCount >= SUSTAINED_PENDING_LIMIT) {
			backgroundSync(boardId);
			return;
		}

		// Condition 3: LOW Activity - Sync at 15 moves (cleanup)
		if (activityLevel === 'low' && pendingMoveCount >= LOW_ACTIVITY_PENDING_LIMIT) {
			backgroundSync(boardId);
			return;
		}

		// Condition 4: MODERATE Activity - Periodic cleanup (reduced frequency)
		if (activityLevel === 'moderate' && pendingMoveCount >= 20 && timeSinceLastSync > 20000) {
			backgroundSync(boardId);
			return;
		}
	};

	// 🔄 Background Sync: Non-blocking sync
	const backgroundSync = async (boardId: string) => {
		if (syncStatus === 'syncing' || syncStatus === 'syncing-bg') return;

		// Get moves to submit
		const movesToSubmit = $moveHistoryStore.get(boardId) || [];
		if (movesToSubmit.length === 0) return;

		// Submit moves without blocking
		syncStatus = 'syncing-bg';
		const moveBatch = getMoveBatchForSubmission(movesToSubmit);

		try {
			// Submit to backend first
			makeMoves(client, moveBatch, boardId);
			lastSyncTime = Date.now();

			// ✅ Immediately flush moves - hash validation will confirm success
			flushMoveHistory(boardId);
			pendingMoveCount = 0;
		} catch (error) {
			console.error('Background sync failed:', error);
			syncStatus = 'failed';
			// Keep moves in queue for retry
			return;
		}
	};

	// 🔄 Background Sync: Handle desync with overlay warning
	const handleDesync = async (backendBoardStr: string) => {
		console.warn('Desync detected - initiating full sync');

		// Pause game and show warning
		isFrozen = true;
		syncStatus = 'desynced';
		overlayMessage = 'Syncing with server... Please wait.';

		// Submit ALL pending moves
		const allPending = flushMoveHistory(boardId!);
		if (allPending.length > 0) {
			try {
				await makeMoves(client, getMoveBatchForSubmission(allPending), boardId!);
			} catch (error) {
				console.error('Failed to submit pending moves during desync:', error);
			}
		}

		// Wait for backend to process
		await new Promise((resolve) => setTimeout(resolve, 1500));

		// Fetch fresh state
		await game.reexecute({ requestPolicy: 'network-only' });

		// Force reconciliation
		if ($game.data?.board?.board) {
			const finalBackendState = boardToString($game.data.board.board);
			const currentLocalState = boardToString(state?.tablet);

			if (finalBackendState !== currentLocalState) {
				// Full reset to backend state
				state = createState($game.data.board.board, 4, boardId!, player);
				validBoardHashes.clear();
				// Add the backend state as valid
				validBoardHashes.add(hashBoard($game.data.board.board));
				lastHashMismatchTime = null;
			}
		}

		// Resume game
		isFrozen = false;
		syncStatus = 'ready';
		pendingMoveCount = 0;
		overlayMessage = undefined;
	};

	// Legacy submit function (used for idle sync and game end)
	const submitMoves = (boardId: string, force = false) => {
		const moves = flushMoveHistory(boardId);
		try {
			if (moves.length > 0 || force) {
				makeMoves(client, getMoveBatchForSubmission(moves), boardId);
				const newTablet = boardToString(state?.tablet);
				stateHash = newTablet ?? '';
				// Don't freeze UI during background sync
				syncStatus = 'syncing-bg';
				pendingMoveCount = 0;
				lastSyncTime = Date.now();
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

	// Reactive polling: Restart polling when boardId changes
	let initGameIntervalId: NodeJS.Timeout | null = null;
	let lastPolledBoardId: string | undefined = undefined;
	
	$: if (boardId !== lastPolledBoardId) {
		// Clear existing interval when boardId changes
		if (initGameIntervalId) {
			clearInterval(initGameIntervalId);
			initGameIntervalId = null;
		}
		
		lastPolledBoardId = boardId;
		
		// Only start polling if we have a boardId
		if (boardId) {
			// Initial fetch
			game.reexecute({ requestPolicy: 'network-only' });
			
		// Start polling interval
		initGameIntervalId = setInterval(() => {
			if (boardId && !$game.data?.board) {
				game.reexecute({ requestPolicy: 'network-only' });
			} else if ($game.data?.board?.boardId === boardId) {
				// Board found AND matches requested boardId, stop polling
				if (initGameIntervalId) {
					clearInterval(initGameIntervalId);
					initGameIntervalId = null;
				}
			} else if ($game.data?.board?.boardId !== boardId) {
				// Wrong board data (stale), keep polling
				game.reexecute({ requestPolicy: 'network-only' });
			}
		}, 500); // Poll every 500ms
		}
	}

	// Lifecycle Hooks
	onMount(() => {
		localBoardId = getBoardId(leaderboardId);
		if (!isMultiplayer && localBoardId && boardId === undefined) {
			boardId = localBoardId;
		}

		const cleanupListeners = setupIdleListener();

		// Offline mode disabled for website

		return () => {
			cleanupListeners();
			if (initGameIntervalId) {
				clearInterval(initGameIntervalId);
			}
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
	let syncingBackgroundStartTime: number | null = null;

	onMount(() => {
		syncIntervalId = setInterval(() => {
			if (offlineMode) return;
			if (!boardId || !$game.data?.board) return;

			// Continuously poll backend state (including inspector mode for live updates)
			game.reexecute({ requestPolicy: 'network-only' });

			// Skip sync validation in inspector mode (no moves to sync)
			if (isInspectorMode) return;

			// If syncing in background, validate using hash lookup
			if (syncStatus === 'syncing-bg' && state?.tablet) {
				if (!syncingBackgroundStartTime) {
					syncingBackgroundStartTime = Date.now();
				}

				const backendBoard = $game.data.board.board;
				const backendHash = hashBoard(backendBoard);

				// ✅ Simple hash validation: Has backend reached a state we've seen?
				if (validBoardHashes.has(backendHash)) {
					// Backend processed our moves successfully
					const localHash = state?.tablet ? hashBoard(state.tablet) : null;
					const backendMatchesLatest = localHash && backendHash === localHash;

					if (backendMatchesLatest && pendingMoveCount === 0) {
						syncStatus = 'ready';
					} else {
						syncStatus = 'synced';
						setTimeout(() => {
							if (pendingMoveCount === 0 && syncStatus === 'synced') {
								syncStatus = 'ready';
							} else if (syncStatus === 'synced') {
								syncStatus = 'pending';
							}
						}, 800);
					}
					isFrozen = false;
					lastHashMismatchTime = null;
					syncingBackgroundStartTime = null;
				} else {
					// No match yet - check if we've been waiting too long
					const syncWaitTime = Date.now() - syncingBackgroundStartTime;

					// Only trigger desync if we've waited > 5 seconds without finding a match
					if (syncWaitTime > 5000) {
						console.warn('Background sync timeout - backend state not found in valid hashes');
						syncingBackgroundStartTime = null;
						const backendBoardStr = boardToString(backendBoard);
						if (backendBoardStr) {
							handleDesync(backendBoardStr);
						}
					}
				}
			} else {
				// Reset sync timer when not syncing
				syncingBackgroundStartTime = null;
			}

			// Simple hash validation: Check if backend state matches any valid state we've seen
			if (state?.tablet) {
				const backendHash = hashBoard($game.data.board.board);

				if (validBoardHashes.has(backendHash)) {
					// ✅ Backend state is valid - reset mismatch tracking
					const localHash = hashBoard(state.tablet);
					const backendMatchesLatest = backendHash === localHash;

					if (backendMatchesLatest && pendingMoveCount === 0) {
						if (syncStatus !== 'ready') {
							syncStatus = 'ready';
						}
					} else if (
						syncStatus !== 'syncing-bg' &&
						syncStatus !== 'syncing' &&
						syncStatus !== 'synced'
					) {
						syncStatus = pendingMoveCount > 0 ? 'pending' : 'ready';
					}
					lastHashMismatchTime = null;
				} else {
					// ❌ Backend state not in our valid hash set
					if (lastHashMismatchTime === null) {
						lastHashMismatchTime = Date.now();
					} else if (Date.now() - lastHashMismatchTime > 15000) {
						// 15 seconds of mismatch
						console.warn(
							'Backend state not found in valid hashes for 15+ seconds - desync detected'
						);
						const backendBoardStr = boardToString($game.data.board.board);
						if (backendBoardStr) {
							handleDesync(backendBoardStr);
						}
					}
				}
			}
		}, 1000); // Check every second

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

		dismissOverlay();
	};

	const playInspectorMoves = () => {
		if (inspectorCurrentMoveIndex >= inspectorMoveHistory.length) {
			inspectorCurrentMoveIndex = 0;
		}
		isInspectorPlaying = true;
		playNextInspectorMove();
	};

	const playNextInspectorMove = () => {
		// Check if we've reached the end of available moves
		if (!isInspectorPlaying || inspectorCurrentMoveIndex >= inspectorMoveHistory.length) {
			stopInspectorPlay();

			// Only show overlay and disable auto-play if the game has actually ended
			const isGameEnded = $game.data?.board?.isEnded;
			if (isGameEnded && inspectorCurrentMoveIndex >= inspectorMoveHistory.length) {
				showOverlayAtEnd();
			}
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
		dismissOverlay();
	};

	const dismissOverlay = () => {
		if (isInspectorMode) {
			hideInspectorOverlay = true;
		}
	};

	const showOverlayAtEnd = () => {
		hideInspectorOverlay = false;
		autoPlayEnabled = false;
	};

	const handleReplayClick = () => {
		// Enable auto-play when replay button is clicked
		if (!autoPlayEnabled) {
			toggleAutoPlay();
		}
	};

	onDestroy(() => {
		setGameCreationStatus(false);
		stopInspectorPlay();
	});

	$: overlayMessage = hideInspectorOverlay
		? undefined
		: $game.data?.board?.player === $userStore.username
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
			onReplayClick={handleReplayClick}
			hideOverlay={hideInspectorOverlay}
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
	{#if $userStore.username && !isInspectorMode}
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
								{syncStatus === 'ready' || syncStatus === 'synced'
											? 'animate-pulse bg-emerald-500'
											: syncStatus === 'pending'
												? 'bg-cyan-400'
												: syncStatus === 'failed'
													? 'bg-red-500'
													: syncStatus === 'desynced'
														? 'animate-pulse bg-red-500'
														: syncStatus === 'syncing-bg' || syncStatus === 'syncing'
															? 'animate-pulse bg-yellow-500'
															: 'bg-surface-400'}"
									></div>
									<span
										class="text-xs capitalize lg:text-sm
								{syncStatus === 'ready' || syncStatus === 'synced'
											? 'text-emerald-400'
											: syncStatus === 'pending'
												? 'text-cyan-400'
												: syncStatus === 'failed'
													? 'text-red-400'
													: syncStatus === 'desynced'
														? 'text-red-400'
														: syncStatus === 'syncing-bg' || syncStatus === 'syncing'
															? 'text-yellow-400'
															: 'text-surface-400'}"
									>
										{syncStatus === 'syncing-bg' ? 'syncing' : syncStatus}
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
					<div
						class="h-2 w-2 rounded-full {offlineMode ? 'bg-orange-500' : 'bg-emerald-500'}"
					></div>
					<span class="whitespace-nowrap text-xs text-surface-400 lg:text-sm"
						>{offlineMode ? 'Go Online' : 'Go Offline'}</span
					>
				</button>
			{/if}
		</div>
	{/if}

	<!-- 🔍 Inspector Mode Controls -->
	{#if isInspectorMode && $game.data?.board}
		<div class="mt-2 rounded-lg border border-purple-500/20 bg-purple-950/20 px-4 py-3">
			<div class="mb-2 flex items-center justify-between">
				<div class="flex items-center gap-2">
					<div class="h-2 w-2 rounded-full bg-purple-400"></div>
					<span class="text-sm font-medium text-purple-400">Inspector</span>
				</div>
				<div class="text-sm font-medium text-surface-300">
					{inspectorCurrentMoveIndex} / {inspectorMoveHistory.length}
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

					// Show overlay only if at end AND game has ended
					const isGameEnded = $game.data?.board?.isEnded;
					if (inspectorCurrentMoveIndex >= inspectorMoveHistory.length && isGameEnded) {
						showOverlayAtEnd();
					} else {
						dismissOverlay();
					}
				}}
				class="inspector-slider w-full"
			/>

			<!-- Playback Controls and Status -->
			<div class="mt-2 flex items-center justify-between gap-3">
				<!-- Buttons -->
				<div class="flex items-center gap-1 sm:gap-2">
					<button
						onclick={restartInspector}
						class="rounded bg-surface-700 px-2 py-1 text-xs text-white transition-colors hover:bg-surface-600 sm:px-3 sm:py-1.5 sm:text-sm"
					>
						Restart
					</button>

					<button
						onclick={toggleAutoPlay}
						class="flex items-center gap-1 rounded text-xs transition-all sm:gap-1.5 sm:text-sm {autoPlayEnabled
							? 'auto-play-active px-[7px] py-[2px] hover:bg-surface-600 sm:px-[9px] sm:py-[3px]'
							: 'border-2 border-surface-700 bg-surface-700 px-[7px] py-[2px] hover:bg-surface-600 sm:border-[3px] sm:px-[9px] sm:py-[3px]'}"
					>
						<div
							class="h-1 w-1 rounded-full sm:h-1.5 sm:w-1.5 {autoPlayEnabled
								? 'bg-emerald-400'
								: 'bg-surface-400'}"
						></div>
						<span class="text-white">Auto-Play</span>
					</button>

					<button
						onclick={() => {
							if (inspectorCurrentMoveIndex < inspectorMoveHistory.length) {
								inspectorCurrentMoveIndex++;
								handleGameStateUpdate();

								// Show overlay only if reached end AND game has ended
								const isGameEnded = $game.data?.board?.isEnded;
								if (inspectorCurrentMoveIndex >= inspectorMoveHistory.length && isGameEnded) {
									showOverlayAtEnd();
								} else {
									dismissOverlay();
								}
							}
						}}
						disabled={inspectorCurrentMoveIndex >= inspectorMoveHistory.length}
						class="rounded bg-surface-700 px-2 py-1 text-xs text-white transition-colors hover:bg-surface-600 disabled:opacity-50 sm:px-3 sm:py-1.5 sm:text-sm"
					>
						Next
					</button>
				</div>

				<!-- Status -->
				{#if autoPlayEnabled}
					<div class="flex items-center gap-1 text-xs sm:gap-1.5 sm:text-sm">
						<span class="text-surface-300">
							{$game.data?.board?.isEnded ? 'Replay' : 'Live'}
						</span>
						<span class="animate-pulse text-purple-400">
							• {inspectorCurrentMoveIndex >= inspectorMoveHistory.length &&
							$game.data?.board?.isEnded
								? 'Ended'
								: isInspectorPlaying
									? 'Playing'
									: 'Waiting'}
						</span>
					</div>
				{:else}
					<div class="truncate text-xs text-surface-300 sm:text-sm">
						{$game.data?.board?.player === $userStore.username
							? 'Replay mode'
							: `Viewing ${$game.data?.board?.player}'s game`}
					</div>
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
		height: 4px;
		border-radius: 2px;
		outline: none;
	}

	.inspector-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 12px;
		height: 12px;
		background: #c084fc;
		cursor: pointer;
		border-radius: 50%;
	}

	.inspector-slider::-moz-range-thumb {
		width: 12px;
		height: 12px;
		background: #c084fc;
		cursor: pointer;
		border-radius: 50%;
		border: none;
	}

	.auto-play-active {
		border: 3px solid;
		background-color: rgb(52 52 55); /* surface-700 */
		animation: border-pulse 2s ease-in-out infinite;
	}

	@keyframes border-pulse {
		0%,
		100% {
			border-color: rgb(16 185 129); /* emerald-500 */
		}
		50% {
			border-color: rgb(52 52 55); /* surface-700 */
		}
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
