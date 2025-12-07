<script lang="ts">
	import { queryStore, gql } from '@urql/svelte';

	import BoardHeader from '../molecules/BoardHeader.svelte';
	import { makeMoves, type MakeMoveResult } from '$lib/graphql/mutations/makeMove';
	import { onDestroy, onMount, createEventDispatcher } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { genInitialState as createState } from '$lib/game/game';
	import type { GameKeys, GameState, Tablet } from '$lib/game/models';
	import { boardSize, setGameCreationStatus } from '$lib/stores/gameStore';
	import { boardToString, computeInitialBoard } from '$lib/game/utils';
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
	import {
		getPaginatedMoveHistory,
		calculateLoadRange,
		type PaginatedMoveHistoryStore,
		type MoveHistoryRecord
	} from '$lib/stores/paginatedMoveHistory';
	import { getBoardPaginated } from '$lib/graphql/queries/getBoardPaginated';
	import { RhythmEngine, type RhythmSettings } from '$lib/game/rhythmEngine.js';
	import BeatIndicator from '../molecules/BeatIndicator.svelte';

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
	export let tournamentDescription: string = '';

	const dispatch = createEventDispatcher();

	// Board ID Management
	let localBoardId: string | null = null;
	let isCreatingNewBoard: boolean = false;

	// GraphQL Definitions
	// Optimized query - excludes moveHistory which is fetched via pagination
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
			}
			balance
		}
	`;

	// State Management
	$: client = getClient(chainId ?? $userStore.chainId);
	let state: GameState | undefined;
	let isInitialized = false;
	let isSynced: boolean = false; // eslint-disable-line @typescript-eslint/no-unused-vars
	let stateHash = ''; // eslint-disable-line @typescript-eslint/no-unused-vars
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
	const MAX_VALID_HASHES = 500; // Limit memory usage - keep only recent states

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
	const hashBoard = (board: Tablet) => {
		let hash = 2166136261;
		for (const row of board) {
			for (const cell of row) {
				const value = cell?.value ?? 0;
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

	// Add board state to valid hashes set (with memory limit)
	const addValidBoardHash = (tablet: Tablet) => {
		if (!tablet) return;
		const hash = hashBoard(tablet);
		
		// Limit memory usage - remove oldest entries if over limit
		if (validBoardHashes.size >= MAX_VALID_HASHES) {
			// Convert to array, keep last half, convert back to set
			const hashArray = Array.from(validBoardHashes);
			validBoardHashes = new Set(hashArray.slice(hashArray.length / 2));
		}
		
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
	let inspectorMoveHistory: MoveHistoryRecord[] = [];
	let inspectorCurrentMoveIndex = 0;
	let isInspectorPlaying = false;
	let inspectorPlayTimeout: ReturnType<typeof setTimeout> | null = null;
	let autoPlayEnabled = false; // Toggle state for auto-play
	// let previousMoveHistoryLength = 0; // Track previous length to detect new moves (no longer needed)
	let hideInspectorOverlay = true; // Control inspector overlay visibility (start hidden)
	let lastInspectedBoardId: string | undefined = undefined; // Track board changes
	let isUserControlledReplay = false; // Flag to prevent auto-positioning during user replay

	// 🚀 Pagination System for move history
	let paginatedHistoryStore: PaginatedMoveHistoryStore | null = null;
	let totalMoves: number = 0;
	let isLoadingMoves: boolean = false;
	let loadingTargetMove: number | undefined = undefined;
	
	// 🎮 Initial board cache for inspector mode (position 0)
	let initialBoardCache: number[][] | null = null;
	let isLoadingInitialBoard: boolean = false;

	// 🎵 Rhythm System
	let rhythmEngine: RhythmEngine | null = null;
	let rhythmSettings: RhythmSettings | null = null;
	let showRhythmIndicator = false;
	let rhythmNeedsStart = false; // True when rhythm mode detected but needs user interaction
	
	// Rhythm scoring
	let rhythmScore = 0;
	let rhythmCombo = 0;
	let maxRhythmCombo = 0;
	let perfectCount = 0;
	let goodCount = 0;
	let missCount = 0;
	let totalRhythmMoves = 0;

	// GraphQL Queries and Subscriptions
	// Note: moveHistory is fetched separately via pagination (getBoardPaginated)
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
		// 🔧 FIX: Also trigger inspector mode for anonymous users viewing any board
		// OR logged-in users viewing someone else's board
		const isOtherPlayer =
			isBoardDataValid && boardPlayer && (
				!currentUser || // Anonymous user viewing any board
				boardPlayer !== currentUser // Logged-in user viewing someone else's board
			);
		const isOwnEndedGame = isBoardDataValid && currentUser && boardPlayer === currentUser && gameEnded;

		if (isOtherPlayer || isOwnEndedGame) {
			// const wasInspectorMode = isInspectorMode; // No longer needed with pagination
			const isBoardChanged = currentBoardId && currentBoardId !== lastInspectedBoardId;

			isInspectorMode = true;
			const newTotalMoves = $game.data?.board?.totalMoves || 0;

			// Initialize pagination store for this board
			if (!paginatedHistoryStore || isBoardChanged) {
				paginatedHistoryStore = getPaginatedMoveHistory(currentBoardId);
				paginatedHistoryStore.initialize(newTotalMoves);
				totalMoves = newTotalMoves;
				
				// Reset initial board cache for new board
				initialBoardCache = null;

				// Load the final batch (most recent moves)
				loadMoveRange(newTotalMoves);

				lastInspectedBoardId = currentBoardId;
			}

			// Update total moves if changed
			if (newTotalMoves !== totalMoves) {
				totalMoves = newTotalMoves;
				paginatedHistoryStore.initialize(newTotalMoves);
			}

			// Set inspector to latest move (unless user is controlling replay)
			if (!isUserControlledReplay) {
				inspectorCurrentMoveIndex = newTotalMoves;
			}
			// No longer tracking previousMoveHistoryLength with pagination
			handleGameStateUpdate();

			// Show overlay if at end and game ended
			if (gameEnded && inspectorCurrentMoveIndex >= totalMoves) {
				hideInspectorOverlay = false;
			} else {
				hideInspectorOverlay = true;
			}
		} else {
			isInspectorMode = false;
			lastInspectedBoardId = undefined;
			isUserControlledReplay = false;
		}
	}

	$: if (isMultiplayer && $game.data?.board === null) {
		goto('/error');
	}

	$: boardEnded = isEnded || $game.data?.board?.isEnded || state?.finished;

	// Debug logging for game end state
	$: if (boardEnded) {
		console.log('🎮 Game ended detected:', {
			isEnded,
			boardIsEnded: $game.data?.board?.isEnded,
			stateFinished: state?.finished,
			boardEnded,
			overlayMessage
		});
	}

	// Calculate loaded ranges reactively for visual indicator
	$: loadedRanges = paginatedHistoryStore ? paginatedHistoryStore.getLoadedRanges() : [];

	// 🎵 Initialize Rhythm System from tournament description
	$: {
		// Use tournament description for rhythm settings (passed from leaderboard)
		const parsedRhythm = RhythmEngine.parseFromDescription(tournamentDescription);
		
		if (parsedRhythm && $game.data?.board && !isInspectorMode) {
			rhythmSettings = parsedRhythm;
			if (!rhythmEngine) {
				// Create engine but don't start yet - wait for user interaction
				rhythmEngine = new RhythmEngine(rhythmSettings);
				rhythmNeedsStart = true;
				showRhythmIndicator = true;
				console.log('🎵 Rhythm mode detected, waiting for user to start:', rhythmSettings);
			} else {
				rhythmEngine.updateSettings(rhythmSettings);
			}
		} else {
			// No rhythm mode or in inspector mode, stop engine if running
			if (rhythmEngine) {
				rhythmEngine.stop();
				rhythmEngine = null;
				showRhythmIndicator = false;
				rhythmSettings = null;
				rhythmNeedsStart = false;
			}
		}
	}

	// 🎵 Start rhythm engine (called on user interaction)
	const startRhythmEngine = async () => {
		if (!rhythmEngine || !rhythmNeedsStart) return;
		
		try {
			await rhythmEngine.initAudio();
			rhythmEngine.start();
			rhythmNeedsStart = false;
			console.log('🎵 Rhythm mode started!');
		} catch (err) {
			console.warn('🎵 Audio initialization failed, visual mode only:', err);
			rhythmEngine.start();
			rhythmNeedsStart = false;
		}
	};

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
			isLeaderboardIdSet = false; // Reset so leaderboardId updates for new board
			
			// 🎵 Reset rhythm stats for new board
			rhythmScore = 0;
			rhythmCombo = 0;
			maxRhythmCombo = 0;
			perfectCount = 0;
			goodCount = 0;
			missCount = 0;
			totalRhythmMoves = 0;
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

		// Inspector mode: use move history data
		if (isInspectorMode && paginatedHistoryStore) {
			if (inspectorCurrentMoveIndex === 0) {
				// 🎮 Position 0: Show initial board (before any moves)
				if (initialBoardCache) {
					state = createState(initialBoardCache, 4, boardId, player);
					score = 0;
				} else {
					// Fallback while loading initial board
					state = createState($game.data?.board?.board, 4, boardId, player);
					score = $game.data?.board?.score || 0;
				}
			} else {
				const currentMove = getCurrentMoveData();
				if (currentMove) {
					state = createState(currentMove.boardAfter, 4, boardId, player);
					score = currentMove.scoreAfter;
				} else {
					// 🔧 FIX: Moves not loaded yet - show current board state as fallback
					// This prevents blank board while moves are loading
					state = createState($game.data?.board?.board, 4, boardId, player);
					score = $game.data?.board?.score || 0;
				}
			}
		} else if (isInspectorMode && !paginatedHistoryStore && $game.data?.board) {
			// 🔧 FIX: Inspector mode but pagination not initialized yet - show current state
			state = createState($game.data?.board?.board, 4, boardId, player);
			score = $game.data?.board?.score || 0;
		} else {
			// Normal mode: use current board state
			state = createState($game.data?.board?.board, 4, boardId, player);
		}

		// Register initial board state as valid to prevent false desync detection
		if (state?.tablet) {
			addValidBoardHash(state.tablet);
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
			if (state?.tablet) {
				addValidBoardHash(state.tablet);
			}

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

	// 🎵 Reference to beat indicator for miss feedback
	let beatIndicatorRef: { showMiss: () => void } | null = null;
	// 🎵 Reference to board for miss effect (shake + flash)
	let boardRef: { triggerMissEffect: () => void } | null = null;

	const handleMove = (direction: GameKeys, timestamp: string) => {
		const now = Date.now();
		if (now - lastMoveTime < MOVE_COOLDOWN) return;
		lastMoveTime = now;

		if (!boardId) return;

		const tournamentEndTime = parseInt($game.data?.board?.endTime || '0');
		if (tournamentEndTime > 0 && tournamentEndTime <= Date.now()) {
			return;
		}

		// 🎵 Auto-start rhythm engine on first move (fallback if overlay bypassed)
		if (rhythmNeedsStart && rhythmEngine) {
			startRhythmEngine();
			return; // Block this move, let them try again after audio starts
		}

		// 🎵 Rhythm Mode Check - BLOCK moves if not on beat
		if (rhythmEngine && rhythmSettings?.enabled) {
			const rhythmFeedback = rhythmEngine.checkRhythm(now);
			
			// Check if move is valid (on beat) - allow perfect, good, early, and late
			// Only 'miss' is blocked (outside tolerance window)
			const isValidMove = rhythmFeedback.accuracy !== 'miss';
			
			if (!isValidMove) {
				// BLOCK the move - show miss feedback
				missCount++;
				rhythmCombo = 0;
				console.log(`❌ BLOCKED! Move not on beat (${Math.abs(rhythmFeedback.timingDiff).toFixed(0)}ms off, tolerance: ${rhythmSettings.tolerance}ms)`);
				
				// Trigger miss visual feedback on beat indicator and board
				beatIndicatorRef?.showMiss();
				boardRef?.triggerMissEffect();
				
				// Don't execute the move
				return;
			}
			
			// Valid move - update stats
			totalRhythmMoves++;
			
			if (rhythmFeedback.accuracy === 'perfect') {
				perfectCount++;
				rhythmCombo++;
				rhythmScore += rhythmFeedback.score * (1 + rhythmCombo * 0.1);
				console.log(`🎵 PERFECT! Combo: ${rhythmCombo} (${rhythmFeedback.timingDiff.toFixed(0)}ms)`);
			} else if (rhythmFeedback.accuracy === 'good') {
				goodCount++;
				rhythmCombo++;
				rhythmScore += rhythmFeedback.score * (1 + rhythmCombo * 0.05);
				console.log(`🎵 GOOD! Combo: ${rhythmCombo} (${rhythmFeedback.timingDiff.toFixed(0)}ms)`);
			} else {
				// early or late - still valid but breaks combo
				console.log(`🎵 ${rhythmFeedback.accuracy.toUpperCase()}! (${rhythmFeedback.timingDiff.toFixed(0)}ms)`);
				// Don't increment combo for early/late, but don't reset it either
			}

			if (rhythmCombo > maxRhythmCombo) {
				maxRhythmCombo = rhythmCombo;
			}
		}

		// Queue move if currently processing, otherwise execute immediately
		if (isProcessingMove) {
			moveQueue.push({ direction, timestamp });
		} else {
			move(boardId, direction);
		}

		dispatch('move', { direction, timestamp });
	};

	let idleTimeout: ReturnType<typeof setTimeout>;
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

	// 🔄 Background Sync: Retry tracking
	let syncRetryCount = 0;
	const MAX_SYNC_RETRIES = 3;
	const SYNC_RETRY_DELAY = 2000; // 2 seconds between retries

	// 🔄 Background Sync: Non-blocking sync with confirmation
	const backgroundSync = async (boardId: string) => {
		if (syncStatus === 'syncing' || syncStatus === 'syncing-bg') return;

		// Get moves to submit (but don't flush yet!)
		const movesToSubmit = $moveHistoryStore.get(boardId) || [];
		if (movesToSubmit.length === 0) return;

		// Mark as syncing
		syncStatus = 'syncing-bg';
		const moveBatch = getMoveBatchForSubmission(movesToSubmit);
		const moveCount = movesToSubmit.length;

		console.log(`🔄 Background sync starting: ${moveCount} moves`);

		try {
			// Submit to backend and WAIT for confirmation
			const result: MakeMoveResult = await makeMoves(client, moveBatch, boardId);

			if (result.success) {
				// ✅ Only flush AFTER confirmed success
				flushMoveHistory(boardId);
				// Sync pendingMoveCount with actual store state (handles race condition where
				// new moves arrived during async submission)
				const remainingMoves = $moveHistoryStore.get(boardId)?.length || 0;
				pendingMoveCount = remainingMoves;
				lastSyncTime = Date.now();
				syncRetryCount = 0; // Reset retry counter on success
				console.log(`✅ Background sync success: ${moveCount} moves synced, ${remainingMoves} remaining`);
				
				// Don't immediately set to ready - let hash validation confirm
				// syncStatus will be updated by the sync interval
			} else {
				// ❌ Submission failed - keep moves for retry
				console.error('Background sync failed:', result.error);
				syncStatus = 'failed';
				syncRetryCount++;
				
				// Schedule retry if under limit
				if (syncRetryCount < MAX_SYNC_RETRIES) {
					console.log(`🔄 Will retry sync in ${SYNC_RETRY_DELAY}ms (attempt ${syncRetryCount + 1}/${MAX_SYNC_RETRIES})`);
					setTimeout(() => {
						if (syncStatus === 'failed') {
							backgroundSync(boardId);
						}
					}, SYNC_RETRY_DELAY);
				} else {
					console.error('❌ Max sync retries exceeded - triggering desync handler');
					handleDesync();
				}
			}
		} catch (error: unknown) {
			console.error('Background sync exception:', error);
			syncStatus = 'failed';
			syncRetryCount++;
			
			// Schedule retry if under limit
			if (syncRetryCount < MAX_SYNC_RETRIES) {
				setTimeout(() => {
					if (syncStatus === 'failed') {
						backgroundSync(boardId);
					}
				}, SYNC_RETRY_DELAY);
			}
		}
	};

	// 🔄 Background Sync: Handle desync with overlay warning
	const handleDesync = async () => {
		console.warn('⚠️ Desync detected - initiating full sync');

		// Pause game and show warning
		isFrozen = true;
		syncStatus = 'desynced';
		overlayMessage = 'Syncing with server... Please wait.';

		// Get ALL pending moves (don't flush yet)
		const allPending = $moveHistoryStore.get(boardId!) || [];
		
		if (allPending.length > 0) {
			console.log(`🔄 Attempting to sync ${allPending.length} pending moves during desync recovery`);
			const result = await makeMoves(client, getMoveBatchForSubmission(allPending), boardId!);
			
			if (result.success) {
				// Only flush after success
				flushMoveHistory(boardId!);
				console.log('✅ Pending moves synced successfully during recovery');
			} else {
				console.error('❌ Failed to submit pending moves during desync:', result.error);
				// Clear the moves anyway to reset state - they're likely invalid
				flushMoveHistory(boardId!);
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
				console.log('🔧 Resetting to backend state');
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
		syncRetryCount = 0; // Reset retry counter
		overlayMessage = undefined;
		console.log('✅ Desync recovery complete');
	};

	// Submit function (used for idle sync and game end)
	const submitMoves = async (boardId: string, force = false) => {
		// Get moves but don't flush yet
		const moves = $moveHistoryStore.get(boardId) || [];
		
		try {
			if (moves.length > 0 || force) {
				syncStatus = 'syncing-bg';
				const moveBatch = getMoveBatchForSubmission(moves);
				const moveCount = moves.length;
				
				console.log(`📤 Submitting ${moveCount} moves (force: ${force})`);
				
				// Wait for confirmation
				const result = await makeMoves(client, moveBatch, boardId);
				
				if (result.success) {
					// Only flush after success
					flushMoveHistory(boardId);
					const newTablet = boardToString(state?.tablet);
					stateHash = newTablet ?? '';
					// Sync pendingMoveCount with actual store state (handles race condition)
					const remainingMoves = $moveHistoryStore.get(boardId)?.length || 0;
					pendingMoveCount = remainingMoves;
					lastSyncTime = Date.now();
					syncRetryCount = 0;
					console.log(`✅ Submit success: ${moveCount} moves, ${remainingMoves} remaining`);
				} else {
					console.error('Submit moves failed:', result.error);
					syncStatus = 'failed';
					// Keep moves for retry - don't restore since we didn't flush
				}
			}
		} catch (error: unknown) {
			console.error('Submit moves exception:', error);
			syncStatus = 'failed';
			// Keep moves for retry - they're still in the store
		} finally {
			activityDetected = false;
		}
	};

	// Offline mode toggle removed for website

	// Add toggle handler for balance view
	let dirtyBalance = false; // eslint-disable-line @typescript-eslint/no-unused-vars
	const toggleBalanceView = () => {
		dirtyBalance = true;
		showBalance = !showBalance;
	};

	const requestFaucet = () => {
		requestFaucetMutation(client);
	};

	// Reactive polling: Restart polling when boardId changes
	let initGameIntervalId: ReturnType<typeof setInterval> | null = null;
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
			// Submit any remaining moves when unmounting (fire-and-forget since we're leaving)
			if (boardId) {
				const moves = $moveHistoryStore.get(boardId) || [];
				if (moves.length > 0) {
					const moveBatch = getMoveBatchForSubmission(moves);
					// Fire and forget on unmount - can't await during cleanup
					makeMoves(client, moveBatch, boardId)
						.then((result) => {
							if (result.success) {
								flushMoveHistory(boardId!);
								console.log('✅ Cleanup: moves synced on unmount');
							}
						})
						.catch((err) => console.error('Cleanup sync failed:', err));
				}
			}
		};
	});

	let syncIntervalId: ReturnType<typeof setInterval>;
	let syncingBackgroundStartTime: number | null = null;

	onMount(() => {
		syncIntervalId = setInterval(() => {
			if (offlineMode) return;
			if (!boardId || !$game.data?.board) return;

			// Continuously poll backend state (including inspector mode for live updates)
			game.reexecute({ requestPolicy: 'network-only' });

			// Skip sync validation in inspector mode (no moves to sync)
			if (isInspectorMode) return;

			const backendBoard = $game.data.board.board;
			if (!backendBoard || !state?.tablet) return;

			const backendHash = hashBoard(backendBoard);
			const localHash = hashBoard(state.tablet);
			const backendMatchesLocal = backendHash === localHash;
			const backendInValidSet = validBoardHashes.has(backendHash);

			// If syncing in background, validate using hash lookup
			if (syncStatus === 'syncing-bg') {
				if (!syncingBackgroundStartTime) {
					syncingBackgroundStartTime = Date.now();
				}

				// ✅ Simple hash validation: Has backend reached a state we've seen?
				if (backendInValidSet) {
					// Backend processed our moves successfully
					if (backendMatchesLocal && pendingMoveCount === 0) {
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

					// Only trigger desync if we've waited > 8 seconds without finding a match
					// Increased from 5s to give backend more time
					if (syncWaitTime > 8000) {
						console.warn('⏱️ Background sync timeout - backend state not found in valid hashes');
						syncingBackgroundStartTime = null;
						handleDesync();
					}
				}
			} else {
				// Reset sync timer when not actively syncing
				syncingBackgroundStartTime = null;
			}

			// Continuous hash validation (less frequent logging)
			if (backendInValidSet) {
				// ✅ Backend state is valid - reset mismatch tracking
				if (backendMatchesLocal && pendingMoveCount === 0) {
					if (syncStatus !== 'ready' && syncStatus !== 'syncing-bg' && syncStatus !== 'syncing') {
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
				// Only start tracking if we have pending moves (otherwise backend is just ahead)
				if (pendingMoveCount > 0 || syncStatus === 'syncing-bg') {
					if (lastHashMismatchTime === null) {
						lastHashMismatchTime = Date.now();
						console.warn('⚠️ Hash mismatch detected - starting 30s timer');
					} else if (Date.now() - lastHashMismatchTime > 30000) {
						// 30 seconds of mismatch
						console.warn('❌ 30+ seconds of hash mismatch - desync detected');
						handleDesync();
					}
				} else {
					// No pending moves and backend is different - likely backend is ahead
					// Accept backend state as valid and add to our hash set
					console.log('📥 Backend ahead - accepting new state');
					validBoardHashes.add(backendHash);
					lastHashMismatchTime = null;
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
		// Continue from current position - don't reset to 0
		// The playNextInspectorMove function will handle stopping at the end
		isInspectorPlaying = true;
		playNextInspectorMove();
	};

	const playNextInspectorMove = async () => {
		// Check if we've reached the end of available moves
		if (!isInspectorPlaying || inspectorCurrentMoveIndex >= totalMoves) {
			stopInspectorPlay();

			// Only show overlay and disable auto-play if the game has actually ended
			const isGameEnded = $game.data?.board?.isEnded;
			if (isGameEnded && inspectorCurrentMoveIndex >= totalMoves) {
				showOverlayAtEnd();
			}
			return;
		}

		// Ensure the next move is loaded
		await loadMoveRange(inspectorCurrentMoveIndex + 1);

		const currentMove = getCurrentMoveData();
		const nextMove = paginatedHistoryStore?.getMove(inspectorCurrentMoveIndex + 1);

		// Calculate delay based on timestamp difference
		let delay = 500; // Default 500ms
		if (currentMove && nextMove) {
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

	const restartInspector = async () => {
		stopInspectorPlay();
		autoPlayEnabled = false;
		isUserControlledReplay = true;
		await handleSliderChange(0); // Start at initial state (before any moves)
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

	const handleReplayClick = async () => {
		// Set user controlled flag to prevent auto-positioning
		isUserControlledReplay = true;

		// Initialize inspector mode if not already active
		if (!isInspectorMode) {
			isInspectorMode = true;

			// Initialize pagination store for this board
			const newTotalMoves = $game.data?.board?.totalMoves || 0;
			if (!paginatedHistoryStore && boardId) {
				paginatedHistoryStore = getPaginatedMoveHistory(boardId);
				paginatedHistoryStore.initialize(newTotalMoves);
				totalMoves = newTotalMoves;
				lastInspectedBoardId = boardId;
			}

			// Load moves if needed
			if (newTotalMoves > 0) {
				await loadMoveRange(newTotalMoves);
			}
		}

		// Start from the beginning for replay (position 0 = initial state)
		inspectorCurrentMoveIndex = 0;
		await handleSliderChange(0);

		// Enable auto-play when replay button is clicked
		if (!autoPlayEnabled) {
			toggleAutoPlay();
		}
	};

	// 🚀 Pagination functions for move history
	const loadMoveRange = async (targetMove: number) => {
		if (!paginatedHistoryStore || isLoadingMoves) return;

		// Check if move is already loaded
		if (paginatedHistoryStore.isMoveLoaded(targetMove)) {
			return;
		}

		// Calculate what range to load
		const { start, limit } = calculateLoadRange(targetMove, totalMoves, 200);

		// Set loading state
		isLoadingMoves = true;
		loadingTargetMove = targetMove;
		paginatedHistoryStore.setLoading(true, targetMove);

		try {
			const boardQuery = getBoardPaginated(
				client,
				boardId,
				start - 1, // Convert to 0-based offset
				limit
			);

			// Wait for the query to complete
			const result = await new Promise<{
				data?: { board?: { moveHistory?: MoveHistoryRecord[] } };
			}>((resolve) => {
				const unsubscribe = boardQuery.subscribe(resolve);
				return unsubscribe;
			});

			if (result?.data?.board?.moveHistory) {
				const moves = result.data.board.moveHistory;
				paginatedHistoryStore.addLoadedRange(start, moves);
				// 🔧 FIX: Update game state after moves are loaded
				handleGameStateUpdate();
			}
		} catch (error: unknown) {
			console.error('Failed to load move range:', error);
		} finally {
			isLoadingMoves = false;
			loadingTargetMove = undefined;
			paginatedHistoryStore.setLoading(false);
		}
	};

	// 🎮 Load and cache the initial board (position 0)
	const loadInitialBoard = async () => {
		if (initialBoardCache || isLoadingInitialBoard) return;
		if (!boardId || !$game.data?.board) return;
		
		isLoadingInitialBoard = true;
		try {
			const boardPlayer = $game.data.board.player;
			const createdAt = $game.data.board.createdAt;
			
			if (boardPlayer && createdAt) {
				initialBoardCache = await computeInitialBoard(boardId, boardPlayer, createdAt);
				// Update display if we're at position 0
				if (inspectorCurrentMoveIndex === 0) {
					handleGameStateUpdate();
				}
			}
		} catch (error) {
			console.error('Failed to compute initial board:', error);
		} finally {
			isLoadingInitialBoard = false;
		}
	};

	const handleSliderChange = async (targetMove: number) => {
		if (!paginatedHistoryStore) return;

		// Set user controlled flag when manually using slider
		isUserControlledReplay = true;

		// Ensure targetMove is within valid bounds (allow 0 for initial state)
		if (targetMove < 0) targetMove = 0;
		if (targetMove > totalMoves) targetMove = totalMoves;

		// Load the target range if not already loaded (skip for position 0)
		if (targetMove > 0) {
			await loadMoveRange(targetMove);
		} else {
			// Position 0: load initial board if needed
			await loadInitialBoard();
		}

		// Update current move index
		inspectorCurrentMoveIndex = targetMove;
		handleGameStateUpdate();

		// Show/hide overlay based on position
		const isGameEnded = $game.data?.board?.isEnded;
		if (isGameEnded && targetMove >= totalMoves) {
			hideInspectorOverlay = false;
		} else {
			hideInspectorOverlay = true;
		}
	};

	// Get current move data for display
	const getCurrentMoveData = () => {
		if (!paginatedHistoryStore || inspectorCurrentMoveIndex === 0) {
			return null;
		}

		return paginatedHistoryStore.getMove(inspectorCurrentMoveIndex);
	};

	onDestroy(() => {
		setGameCreationStatus(false);
		stopInspectorPlay();
		// 🎵 Clean up rhythm engine to stop audio
		if (rhythmEngine) {
			rhythmEngine.stop();
			rhythmEngine = null;
		}
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
			bind:this={boardRef}
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
				<div class="board-header-content">
					<BoardHeader
						bind:boardId
						bind:isCreating={isCreatingNewBoard}
						{canStartNewGame}
						{showBestScore}
						player={$game.data?.board?.player ?? $userStore.username}
						{score}
						{bestScore}
					/>
					{#if showRhythmIndicator && rhythmEngine}
						<div class="rhythm-indicator-wrapper">
							<BeatIndicator 
								{rhythmEngine} 
								bind:this={beatIndicatorRef}
							/>
						</div>
						<div class="rhythm-stats">
							<span class="stat">
								<span class="label">Combo</span>
								<span class="value text-green-400">{rhythmCombo}</span>
							</span>
							<span class="stat">
								<span class="label">Moves</span>
								<span class="value text-purple-400">{totalRhythmMoves}</span>
							</span>
							<span class="stat">
								<span class="label">Miss</span>
								<span class="value text-red-400">{missCount}</span>
							</span>
							{#if rhythmEngine?.isUsingMusic()}
								<span class="stat">
									<span class="label">BPM</span>
									<span class="value text-yellow-400">{rhythmEngine.getDetectedBpm() || '?'}</span>
								</span>
							{/if}
						</div>
					{/if}
				</div>
			{/snippet}
		</Board>
		
		<!-- 🎵 Rhythm Start Overlay - requires user interaction to start audio -->
		{#if rhythmNeedsStart && !isInspectorMode}
			<button
				class="rhythm-start-overlay"
				onclick={startRhythmEngine}
				aria-label="Start rhythm mode"
			>
				<div class="rhythm-start-content">
					<div class="rhythm-icon">🎵</div>
					<div class="rhythm-title">Rhythm Mode</div>
					<div class="rhythm-subtitle">Tap to Start Music</div>
				</div>
			</button>
		{/if}
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
					{inspectorCurrentMoveIndex} / {totalMoves}
					{#if inspectorCurrentMoveIndex === 0}
						<span class="text-xs text-green-400">(initial)</span>
					{:else if isLoadingMoves || isLoadingInitialBoard}
						<span class="text-xs text-purple-400">(loading...)</span>
					{/if}
				</div>
			</div>

			<!-- Progress Slider with Loaded Ranges Indicator -->
			<div class="relative">
				<!-- Loaded ranges visual indicator -->
				<div class="pointer-events-none absolute left-0 top-0 h-1 w-full">
					{#each loadedRanges as range}
						<div
							class="absolute h-full bg-purple-500/30"
							style="left: {((range.start - 1) / totalMoves) * 100}%; width: {((range.end -
								range.start +
								1) /
								totalMoves) *
								100}%"
						></div>
					{/each}
				</div>

				<input
					type="range"
					min="0"
					max={totalMoves}
					value={inspectorCurrentMoveIndex}
					oninput={(e) => {
						stopInspectorPlay();
						const target = e.target as HTMLInputElement;
						const targetMove = parseInt(target.value);
						handleSliderChange(targetMove);
					}}
					disabled={isLoadingMoves || isLoadingInitialBoard}
					class="inspector-slider w-full {isLoadingMoves || isLoadingInitialBoard ? 'opacity-50' : ''}"
				/>

				{#if isLoadingMoves}
					<div class="pointer-events-none absolute inset-0 flex items-center justify-center">
						<div class="rounded bg-surface-800/90 px-2 py-1 text-xs text-purple-400">
							Loading moves {loadingTargetMove || ''}...
						</div>
					</div>
				{/if}
			</div>

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
						onclick={async () => {
							if (inspectorCurrentMoveIndex > 0) {
								const targetMove = inspectorCurrentMoveIndex - 1;
								await handleSliderChange(targetMove);
							}
						}}
						disabled={inspectorCurrentMoveIndex <= 0 || isLoadingMoves || isLoadingInitialBoard}
						class="rounded bg-surface-700 px-2 py-1 text-xs text-white transition-colors hover:bg-surface-600 disabled:opacity-50 sm:px-3 sm:py-1.5 sm:text-sm"
					>
						Previous
					</button>

					<button
						onclick={async () => {
							if (inspectorCurrentMoveIndex < totalMoves) {
								const targetMove = inspectorCurrentMoveIndex + 1;
								await handleSliderChange(targetMove);
							}
						}}
						disabled={inspectorCurrentMoveIndex >= totalMoves || isLoadingMoves}
						class="rounded bg-surface-700 px-2 py-1 text-xs text-white transition-colors hover:bg-surface-600 disabled:opacity-50 sm:px-3 sm:py-1.5 sm:text-sm"
					>
						Next
					</button>
				</div>

				<!-- Status -->
				{#if autoPlayEnabled}
					<div class="flex flex-col gap-1 text-xs sm:gap-1.5 sm:text-sm">
						<div class="flex items-center gap-1">
							<span class="text-surface-300">
								{$game.data?.board?.isEnded ? 'Replay' : 'Live'}
							</span>
							<span class="animate-pulse text-purple-400">
								• {inspectorCurrentMoveIndex >= totalMoves && $game.data?.board?.isEnded
									? 'Ended'
									: isInspectorPlaying
										? 'Playing'
										: isLoadingMoves
											? 'Loading...'
											: 'Waiting'}
							</span>
						</div>
					</div>
				{:else}
					<div class="flex flex-col gap-1">
						<div class="truncate text-xs text-surface-300 sm:text-sm">
							{$game.data?.board?.player === $userStore.username
								? 'Replay mode'
								: `Viewing ${$game.data?.board?.player}'s game`}
						</div>
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

	/* Rhythm Indicator Styles */
	.board-header-content {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		width: 100%;
	}

	.rhythm-indicator-wrapper {
		display: flex;
		justify-content: center;
		padding: 0.75rem 0;
		border-top: 1px solid rgba(139, 92, 246, 0.3);
		margin-top: 0.5rem;
		background: linear-gradient(180deg, rgba(139, 92, 246, 0.1) 0%, transparent 100%);
	}

	.rhythm-stats {
		display: flex;
		justify-content: center;
		gap: 1.5rem;
		padding: 0.5rem 0;
		font-size: 0.875rem;
	}

	.rhythm-stats .stat {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.125rem;
	}

	.rhythm-stats .label {
		font-size: 0.625rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: #9ca3af;
	}

	.rhythm-stats .value {
		font-weight: bold;
		font-size: 1rem;
	}

	/* Responsive adjustments for rhythm indicator */
	@media (max-width: 640px) {
		.rhythm-indicator-wrapper {
			padding: 0.5rem 0;
			margin-top: 0.25rem;
		}

		.rhythm-stats {
			gap: 1rem;
			font-size: 0.75rem;
		}

		.rhythm-stats .value {
			font-size: 0.875rem;
		}
	}

	/* 🎵 Rhythm Start Overlay */
	.rhythm-start-overlay {
		position: absolute;
		inset: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.85);
		backdrop-filter: blur(4px);
		z-index: 50;
		cursor: pointer;
		border: none;
		border-radius: 8px;
		animation: rhythm-pulse 2s ease-in-out infinite;
	}

	.rhythm-start-overlay:hover {
		background: rgba(0, 0, 0, 0.75);
	}

	@keyframes rhythm-pulse {
		0%, 100% { box-shadow: 0 0 20px rgba(139, 92, 246, 0.5); }
		50% { box-shadow: 0 0 40px rgba(139, 92, 246, 0.8); }
	}

	.rhythm-start-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
	}

	.rhythm-icon {
		font-size: 3rem;
		animation: bounce 1s ease-in-out infinite;
	}

	@keyframes bounce {
		0%, 100% { transform: translateY(0); }
		50% { transform: translateY(-10px); }
	}

	.rhythm-title {
		font-size: 1.5rem;
		font-weight: bold;
		color: #a78bfa;
	}

	.rhythm-subtitle {
		font-size: 1rem;
		color: #9ca3af;
	}

	@media (max-width: 640px) {
		.rhythm-icon {
			font-size: 2.5rem;
		}

		.rhythm-title {
			font-size: 1.25rem;
		}

		.rhythm-subtitle {
			font-size: 0.875rem;
		}
	}
</style>
