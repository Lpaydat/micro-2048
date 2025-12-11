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
		flushNMoves,
		flushMovesUpToTimestamp,
		getMoveBatchForSubmission
	} from '$lib/stores/moveHistories';
	import { formatBalance } from '$lib/utils/formatBalance';
	import { requestFaucetMutation } from '$lib/graphql/mutations/requestFaucet';
	import { submitCurrentScore } from '$lib/graphql/mutations/submitCurrentScore';
	import {
		getPaginatedMoveHistory,
		calculateLoadRange,
		type PaginatedMoveHistoryStore,
		type MoveHistoryRecord
	} from '$lib/stores/paginatedMoveHistory';
	import { getBoardPaginated } from '$lib/graphql/queries/getBoardPaginated';
	import { RhythmEngine, MUSIC_TRACKS, type RhythmSettings } from '$lib/game/rhythmEngine.js';
	import BeatIndicator from '../molecules/BeatIndicator.svelte';
	import RhythmCalibrationModal from '../molecules/RhythmCalibrationModal.svelte';
	import { getModalStore } from '@skeletonlabs/skeleton';

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
	const modalStore = getModalStore();

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
	
	// 🔧 FIX: Track when we're waiting for backend to process submitted moves
	// Mutation "success" only means "accepted", not "processed"
	// This flag prevents premature state reset while backend is processing
	let awaitingBackendSync = false;
	
	// 🔒 DUPLICATE PREVENTION: Track the latest timestamp that was submitted
	// When backend hash matches local, we flush all moves up to this timestamp
	// This prevents flushing the wrong moves when multiple submissions overlap
	let lastSubmittedTimestamp: number = 0; // Latest move timestamp we've submitted
	
	// 🔒 SYNC LOCK: Prevent concurrent submissions
	// Only one submission should be in-flight at a time
	let isSyncInFlight = false;

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
	let needsReinitAfterInspector = false; // 🔧 FIX: Track if we need to reinit when data arrives

	// 🚀 Pagination System for move history
	let paginatedHistoryStore: PaginatedMoveHistoryStore | null = null;
	let totalMoves: number = 0;
	let isLoadingMoves: boolean = false;
	let loadingTargetMove: number | undefined = undefined;
	
	// 🎮 Initial board cache for inspector mode (position 0)
	let initialBoardCache: number[][] | null = null;
	let isLoadingInitialBoard: boolean = false;

	// 🔍 Board not found tracking
	let boardNotFoundCount = 0;
	let isBoardNotFound = false;
	let boardRedirectCountdown = 5;
	let boardRedirectCountdownInterval: ReturnType<typeof setInterval> | null = null; // Track interval for cleanup
	let boardCreationTime: number | null = null; // Track when we started looking for this board
	const MAX_BOARD_NOT_FOUND_ATTEMPTS = 20; // Stop after 20 consecutive not found (~10 seconds at 500ms)
	const NEW_BOARD_GRACE_PERIOD = 15000; // 15 seconds grace period for newly created boards

	// 🎵 Rhythm System
	let rhythmEngine: RhythmEngine | null = null;
	let rhythmSettings: RhythmSettings | null = null;
	let showRhythmIndicator = false;
	let rhythmNeedsStart = false; // True when rhythm mode detected but needs user interaction
	let displayBpm = 0; // Reactive BPM for display (updated after engine starts)
	let displayTrackName = ''; // Track name for display
	let hasCalibration = false; // Whether user has calibrated
	let calibrationOffset = 0; // Current calibration offset
	
	// Function to refresh calibration status
	const refreshCalibrationStatus = () => {
		if (typeof window !== 'undefined') {
			hasCalibration = RhythmEngine.hasCalibration();
			calibrationOffset = RhythmEngine.getStoredCalibration();
		}
	};
	
	// Check calibration status on mount
	$: if (typeof window !== 'undefined') {
		refreshCalibrationStatus();
	}
	
	// Rhythm scoring
	let rhythmScore = 0;
	let rhythmCombo = 0;
	let maxRhythmCombo = 0;
	let perfectCount = 0;
	let goodCount = 0;
	let missCount = 0;
	let totalRhythmMoves = 0;
	
	// 🎵 Beat phase for board pulse animation (-1 = disabled)
	let beatPhase = -1;
	let beatPhaseAnimationId: number | null = null;

	// GraphQL Queries and Subscriptions
	// Note: moveHistory is fetched separately via pagination (getBoardPaginated)
	$: game = queryStore({
		client,
		query: GET_BOARD_STATE,
		variables: { boardId },
		requestPolicy: 'network-only'
	});

	// Reactive Statements
	// 🔧 FIX: Only update score from game data when NOT in inspector mode
	// During replay, score is managed by handleGameStateUpdate() based on move history
	$: if (!isInspectorMode) {
		score = $game.data?.board?.score || 0;
	}

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
				// Only update game state when not in user-controlled replay
				// This prevents the fallback from showing final board during active replay
				handleGameStateUpdate();
			}

			// Show overlay if at end and game ended
			if (gameEnded && inspectorCurrentMoveIndex >= totalMoves) {
				hideInspectorOverlay = false;
			} else {
				hideInspectorOverlay = true;
			}
		} else {
			// 🔧 FIX: When exiting inspector mode, need to reinitialize board state
			// Set flag if we were in inspector mode - will reinit when data is valid
			if (isInspectorMode) {
				needsReinitAfterInspector = true;
			}
			
			isInspectorMode = false;
			lastInspectedBoardId = undefined;
			isUserControlledReplay = false;
			
			// If we need to reinit and data is now valid, do it
			if (needsReinitAfterInspector && isBoardDataValid) {
				needsReinitAfterInspector = false;
				handleGameStateUpdate();
			}
		}
	}

	$: if (isMultiplayer && $game.data?.board === null) {
		goto('/error');
	}

	$: boardEnded = isEnded || $game.data?.board?.isEnded || state?.finished;



	// Calculate loaded ranges reactively for visual indicator
	$: loadedRanges = paginatedHistoryStore ? paginatedHistoryStore.getLoadedRanges() : [];

	// 🎵 Initialize Rhythm System from tournament description
	$: {
		// Use tournament description for rhythm settings (passed from leaderboard)
		const parsedRhythm = RhythmEngine.parseFromDescription(tournamentDescription);
		
		
		// Initialize rhythm mode when:
		// 1. Rhythm settings are detected in description
		// 2. Not in inspector mode
		// Note: We DON'T require $game.data?.board - rhythm UI should show immediately
		if (parsedRhythm && !isInspectorMode) {
			rhythmSettings = parsedRhythm;
			if (!rhythmEngine) {
				// Create engine but don't start yet - wait for user interaction
				rhythmEngine = new RhythmEngine(rhythmSettings);
				rhythmNeedsStart = true;
				showRhythmIndicator = true;

			}
			// Note: New engine doesn't support updateSettings - settings are immutable
			// If settings change, we'd need to recreate the engine
		} else if (!parsedRhythm) {
			// No rhythm mode, stop engine if running
			if (rhythmEngine) {

				rhythmEngine.stop();
				rhythmEngine = null;
				showRhythmIndicator = false;
				rhythmSettings = null;
				rhythmNeedsStart = false;
			}
		}
		// Note: If in inspector mode but rhythm was detected, we keep the engine
		// stopped but don't destroy it (in case user switches back)
	}

	// 🎵 Open calibration modal
	const openCalibrationModal = () => {
		modalStore.trigger({
			type: 'component',
			component: { ref: RhythmCalibrationModal },
			response: (result: unknown) => {
				// Refresh calibration status after modal closes
				refreshCalibrationStatus();
			}
		});
	};

	// 🎵 Start rhythm engine (called on user interaction)
	const startRhythmEngine = async () => {
		if (!rhythmEngine || !rhythmNeedsStart) return;
		
		try {

			await rhythmEngine.init();

			rhythmEngine.start();
			
			// Update reactive values for display
			displayBpm = rhythmEngine.getBpm();
			const track = rhythmEngine.getCurrentTrack();
			displayTrackName = track?.name || '';
			
			// 🎵 Start beat phase animation loop for board pulse
			startBeatPhaseAnimation();
			
			rhythmEngine.debugState();
			rhythmNeedsStart = false;

		} catch (err) {
			console.warn('🎵 Audio initialization failed:', err);
			rhythmNeedsStart = false;
		}
	};
	
	// 🎵 Animation loop for beat phase (board pulse)
	const startBeatPhaseAnimation = () => {
		if (beatPhaseAnimationId !== null) return; // Already running
		
		const animate = () => {
			if (rhythmEngine?.isRunning()) {
				beatPhase = rhythmEngine.getBeatPhase();
				beatPhaseAnimationId = requestAnimationFrame(animate);
			} else {
				beatPhase = -1;
				beatPhaseAnimationId = null;
			}
		};
		animate();
	};
	
	// 🎵 Stop beat phase animation
	const stopBeatPhaseAnimation = () => {
		if (beatPhaseAnimationId !== null) {
			cancelAnimationFrame(beatPhaseAnimationId);
			beatPhaseAnimationId = null;
		}
		beatPhase = -1;
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
		$game.data?.board?.boardId === boardId && // 🔧 FIX: Ensure data matches requested board (prevent stale data)
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
			lastSubmittedTimestamp = 0; // 🔒 Reset submitted timestamp tracking
			isSyncInFlight = false; // 🔒 Reset sync lock
			syncStatus = 'ready';
			lastHashMismatchTime = null;
			lastBoardId = boardId;
			isLeaderboardIdSet = false; // Reset so leaderboardId updates for new board
			lastUsedTimestamp = 0; // Reset timestamp tracking for new board
			moveQueue = []; // Clear any queued moves from previous board
			awaitingBackendSync = false; // 🔒 Reset awaiting state
			
			// 🔧 FIX: Reset inspector mode related state when switching boards
			// Note: Don't set isInspectorMode directly here - let the reactive block at line ~287 handle it
			// Just clear the cached state so inspector mode re-evaluates correctly
			paginatedHistoryStore = null;
			lastInspectedBoardId = undefined;
			isUserControlledReplay = false;
			inspectorCurrentMoveIndex = 0;
			totalMoves = 0;
			initialBoardCache = null;
			inspectorMoveHistory = [];
			
			// 🎵 Reset rhythm stats for new board
			rhythmScore = 0;
			rhythmCombo = 0;
			maxRhythmCombo = 0;
			perfectCount = 0;
			goodCount = 0;
			missCount = 0;
			displayBpm = 0;
			totalRhythmMoves = 0;
		}
		handleGameStateUpdate();
	}

	// Handle tournament ending while player was offline
	$: if ($game.data?.board && $game.data?.board?.boardId === boardId && boardId && player && !isInspectorMode && isInitialized) {
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
		// Skip game over check in inspector mode - we manage finished state based on replay position
		if (isInspectorMode && paginatedHistoryStore) {
			if (inspectorCurrentMoveIndex === 0) {
				// 🎮 Position 0: Show initial board (before any moves)
				if (initialBoardCache) {
					state = createState(initialBoardCache, 4, boardId, player, true);
					score = 0;
				} else {
					// Fallback while loading initial board
					state = createState($game.data?.board?.board, 4, boardId, player, true);
					score = $game.data?.board?.score || 0;
				}
			} else {
				const currentMove = getCurrentMoveData();
				if (currentMove) {
					state = createState(currentMove.boardAfter, 4, boardId, player, true);
					score = currentMove.scoreAfter;
				} else {
					// 🔧 FIX: Moves not loaded yet - show current board state as fallback
					// This prevents blank board while moves are loading
					state = createState($game.data?.board?.board, 4, boardId, player, true);
					score = $game.data?.board?.score || 0;
				}
			}
		} else if (isInspectorMode && !paginatedHistoryStore && $game.data?.board) {
			// 🔧 FIX: Inspector mode but pagination not initialized yet - show current state
			state = createState($game.data?.board?.board, 4, boardId, player, true);
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

	// Track last used timestamp to ensure strictly increasing order
	let lastUsedTimestamp = 0;

	// Movement Functions
	const move = async (boardId: string, direction: GameKeys, inputTimestamp?: string) => {
		if (!canMakeMove || boardEnded || !state || isProcessingMove) return;

		const tournamentEndTime = parseInt($game.data?.board?.endTime || '0');
		if (tournamentEndTime > 0 && tournamentEndTime <= Date.now()) {
			return;
		}

		isProcessingMove = true;

		try {
			// 🔧 FIX: Use the input timestamp if provided, otherwise generate new one
			// Ensure timestamp is strictly increasing to prevent backend rejection
			let timestampNum = inputTimestamp ? parseInt(inputTimestamp) : Date.now();
			if (timestampNum <= lastUsedTimestamp) {
				timestampNum = lastUsedTimestamp + 1;
			}
			lastUsedTimestamp = timestampNum;
			const timestamp = timestampNum.toString();

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
			
			// 🔍 DEBUG: Log move being added to history

			
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
			// 🔧 FIX: Pass the original timestamp from when user pressed the key
			await move(boardId, nextMove.direction, nextMove.timestamp);
		}
	};

	let lastMoveTime = 0;
	const MOVE_COOLDOWN = 80; // 80ms minimum between moves (prevents timestamp collisions)

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
			const rhythmFeedback = rhythmEngine.checkRhythm();
			
			// Check if move is valid (on beat) - allow perfect, good, early, and late
			// Only 'miss' is blocked (outside tolerance window)
			const isValidMove = rhythmFeedback.accuracy !== 'miss';
			
			if (!isValidMove) {
				// BLOCK the move - show miss feedback
				missCount++;
				rhythmCombo = 0;

				
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

			} else if (rhythmFeedback.accuracy === 'good') {
				goodCount++;
				rhythmCombo++;
				rhythmScore += rhythmFeedback.score * (1 + rhythmCombo * 0.05);

			} else {
				// early or late - still valid but breaks combo

				// Don't increment combo for early/late, but don't reset it either
			}

			if (rhythmCombo > maxRhythmCombo) {
				maxRhythmCombo = rhythmCombo;
			}
		}

		// Queue move if currently processing, otherwise execute immediately
		// 🔧 FIX: Always pass the timestamp to preserve timing information
		if (isProcessingMove) {

			moveQueue.push({ direction, timestamp });
		} else {
			move(boardId, direction, timestamp);
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
		// 🔒 SYNC LOCK: Prevent concurrent submissions
		// This is the PRIMARY lock - if a sync is in-flight, don't start another
		if (isSyncInFlight) {
			console.log('🔒 Sync already in-flight, skipping');
			return;
		}
		
		if (syncStatus === 'desynced') {
			return;
		}

		// Get moves to submit
		const movesToSubmit = [...($moveHistoryStore.get(boardId) || [])];
		if (movesToSubmit.length === 0) return;

		// 🔒 ACQUIRE LOCK immediately before any async operation
		isSyncInFlight = true;
		syncStatus = 'syncing-bg';
		
		// Get the latest timestamp from the batch we're submitting
		const latestTimestampInBatch = Math.max(
			...movesToSubmit.map(m => parseInt(m.timestamp))
		);
		
		const moveBatch = getMoveBatchForSubmission(movesToSubmit);
		const moveCount = movesToSubmit.length;
		
		// 🔧 FIX: Mark that we're waiting for backend to process submitted moves
		awaitingBackendSync = true;

		try {
			// Submit to backend and WAIT for confirmation (note: "success" = accepted, not processed)
			const result: MakeMoveResult = await makeMoves(client, moveBatch, boardId);

			if (result.success) {
				// 🔧 FIX: Optimistic flush - if backend accepted, trust it will process
				// During peak load, hash confirmation may never come, causing infinite resubmission
				const flushedCount = flushMovesUpToTimestamp(boardId, latestTimestampInBatch);
				console.log(`📤 Submitted ${moveCount} moves, flushed ${flushedCount} (latest ts: ${latestTimestampInBatch})`);
				
				// Update pending count
				const remainingMoves = $moveHistoryStore.get(boardId)?.length || 0;
				pendingMoveCount = remainingMoves;
				
				lastSyncTime = Date.now();
				syncRetryCount = 0;
				lastSubmittedTimestamp = 0; // Reset - we flushed optimistically
				awaitingBackendSync = false;
				
				// Transition status based on remaining moves
				if (remainingMoves > 0) {
					syncStatus = 'pending';
				} else {
					syncStatus = 'synced';
					setTimeout(() => {
						if (syncStatus === 'synced') {
							syncStatus = 'ready';
						}
					}, 500);
				}
			} else {
				// Submission failed - keep moves for retry
				console.error('Background sync failed:', result.error);
				syncStatus = 'failed';
				syncRetryCount++;
				awaitingBackendSync = false;
				
				// Schedule retry if under limit
				if (syncRetryCount < MAX_SYNC_RETRIES) {
					setTimeout(() => {
						if (syncStatus === 'failed') {
							backgroundSync(boardId);
						}
					}, SYNC_RETRY_DELAY);
				} else {
					console.error('Max sync retries exceeded');
					handleDesync();
				}
			}
		} catch (error: unknown) {
			console.error('Background sync exception:', error);
			syncStatus = 'failed';
			syncRetryCount++;
			awaitingBackendSync = false;
			
			// Schedule retry if under limit
			if (syncRetryCount < MAX_SYNC_RETRIES) {
				setTimeout(() => {
					if (syncStatus === 'failed') {
						backgroundSync(boardId);
					}
				}, SYNC_RETRY_DELAY);
			}
		} finally {
			// 🔒 RELEASE LOCK - always release even on error
			isSyncInFlight = false;
		}
	};

	// 🔄 Background Sync: Handle desync with overlay warning
	const handleDesync = async () => {
		// 🔧 FIX: Don't desync if game is already finished - preserve game over state
		if (state?.finished || boardEnded) {

			return;
		}
		
		// Pause game and show warning
		isFrozen = true;
		syncStatus = 'desynced';
		overlayMessage = 'Syncing with server... Please wait.';

		// Get ALL pending moves (don't flush yet - wait for backend to process)
		const allPending = $moveHistoryStore.get(boardId!) || [];
		const pendingCount = allPending.length;
		
		if (pendingCount > 0) {

			const result = await makeMoves(client, getMoveBatchForSubmission(allPending), boardId!);
			
			if (!result.success) {
				console.error('Failed to submit pending moves during desync:', result.error);
				// Don't flush on failure - keep moves for retry
			}
			// 🔧 FIX: DON'T flush here - wait for backend to process in the polling loop
			// Moves will be flushed when backend state matches local state
		}

		// 🔧 FIX: Wait longer for backend to process during peak load
		// Poll multiple times instead of single wait
		const MAX_POLL_ATTEMPTS = 5;
		const POLL_INTERVAL = 1500; // 1.5s between polls
		
		for (let attempt = 0; attempt < MAX_POLL_ATTEMPTS; attempt++) {
			await new Promise((resolve) => setTimeout(resolve, POLL_INTERVAL));
			await game.reexecute({ requestPolicy: 'network-only' });
			
			// Check if backend state is now valid (matches one of our known states)
			if ($game.data?.board?.board) {
				const backendHash = hashBoard($game.data.board.board);
				if (validBoardHashes.has(backendHash)) {

					break;
				}
			}
			
			if (attempt < MAX_POLL_ATTEMPTS - 1) {

			}
		}

		// Force reconciliation
		if ($game.data?.board?.board) {
			const backendHash = hashBoard($game.data.board.board);
			const backendIsValid = validBoardHashes.has(backendHash);
			const localHash = state?.tablet ? hashBoard(state.tablet) : null;
			const remainingMoves = $moveHistoryStore.get(boardId!) || [];
			
			if (backendHash === localHash) {
				// ✅ Perfect match - backend caught up to our local state
				console.log('✅ handleDesync: Backend matched local state, flushing all moves');
				flushMoveHistory(boardId!);
				lastSubmittedTimestamp = 0;
			} else if (backendIsValid && remainingMoves.length === 0) {
				// Backend is at valid state and no pending moves - sync with backend
				console.log('✅ handleDesync: Backend at valid state, syncing');
				const newState = createState($game.data.board.board, 4, boardId!, player);
				if ($game.data.board.isEnded) {
					newState.finished = true;
				}
				state = newState;
				score = $game.data.board.score || 0;
				lastSubmittedTimestamp = 0;
			} else if (remainingMoves.length > 0) {
				// We have pending moves - DON'T reset, keep local state
				// Backend will eventually process our submitted moves

			} else {
				// Backend has unknown state and no pending moves - true desync, reset
				console.error('🔄 handleDesync: TRUE DESYNC - resetting to backend state', {
					backendHash,
					validHashCount: validBoardHashes.size,
					localHash
				});
				
				const newState = createState($game.data.board.board, 4, boardId!, player);
				if ($game.data.board.isEnded) {
					newState.finished = true;
				}
				state = newState;
				score = $game.data.board.score || 0;
				validBoardHashes.clear();
				validBoardHashes.add(backendHash);
			}
			lastHashMismatchTime = null;
		}

		// Resume game
		isFrozen = false;
		syncStatus = 'ready';
		pendingMoveCount = 0;
		lastSubmittedTimestamp = 0; // 🔒 Reset submitted timestamp
		isSyncInFlight = false; // 🔒 Release sync lock
		syncRetryCount = 0;
		overlayMessage = undefined;
		
		// 🔧 FIX: Clear awaiting state after desync handling
		awaitingBackendSync = false;
		
	};

	// Submit function (used for idle sync and game end)
	const submitMoves = async (boardId: string, force = false) => {
		// 🔒 SYNC LOCK: Prevent concurrent submissions
		if (isSyncInFlight) {
			console.log('🔒 Sync already in-flight, skipping submitMoves');
			return;
		}
		
		if (syncStatus === 'desynced') {
			return;
		}

		// Take a snapshot of moves to submit (preserves moves added during sync)
		const moves = [...($moveHistoryStore.get(boardId) || [])];
		
		// Early return if no moves to submit (unless forced for game end)
		if (moves.length === 0 && !force) {
			return;
		}

		// 🔒 ACQUIRE LOCK immediately before any async operation
		isSyncInFlight = true;
		syncStatus = 'syncing-bg';
		
		// Get the latest timestamp from the batch we're submitting
		const latestTimestampInBatch = moves.length > 0 
			? Math.max(...moves.map(m => parseInt(m.timestamp)))
			: 0;
		
		const moveBatch = getMoveBatchForSubmission(moves);
		const moveCount = moves.length;
		
		// 🔧 FIX: Mark that we're waiting for backend to process submitted moves
		awaitingBackendSync = true;
		
		try {
			// Wait for confirmation (note: "success" = accepted, not processed)
			const result = await makeMoves(client, moveBatch, boardId);
			
			if (result.success) {
				// 🔧 FIX: Optimistic flush - if backend accepted, trust it will process
				const flushedCount = flushMovesUpToTimestamp(boardId, latestTimestampInBatch);
				console.log(`📤 Submitted ${moveCount} moves via submitMoves, flushed ${flushedCount} (latest ts: ${latestTimestampInBatch})`);
				
				// Update pending count
				const remainingMoves = $moveHistoryStore.get(boardId)?.length || 0;
				pendingMoveCount = remainingMoves;
				
				const newTablet = boardToString(state?.tablet);
				stateHash = newTablet ?? '';
				lastSyncTime = Date.now();
				syncRetryCount = 0;
				lastSubmittedTimestamp = 0; // Reset - we flushed optimistically
				awaitingBackendSync = false;
				
				// Transition status based on remaining moves
				if (remainingMoves > 0) {
					syncStatus = 'pending';
				} else {
					syncStatus = 'synced';
					setTimeout(() => {
						if (syncStatus === 'synced') {
							syncStatus = 'ready';
						}
					}, 500);
				}
			} else {
				console.error('Submit moves failed:', result.error);
				syncStatus = 'failed';
				awaitingBackendSync = false;
			}
		} catch (error: unknown) {
			console.error('Submit moves exception:', error);
			syncStatus = 'failed';
			awaitingBackendSync = false;
		} finally {
			// 🔒 RELEASE LOCK - always release even on error
			isSyncInFlight = false;
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

	// 🚀 Manual Score Submission to Leaderboard
	let isSubmittingScore = false;
	let scoreSubmitCooldownRemaining = 0;
	let scoreAlreadyBest = false; // Show "Already Best" feedback
	let scoreSubmitCooldownInterval: ReturnType<typeof setInterval> | null = null; // Track interval for cleanup
	const SCORE_SUBMIT_COOLDOWN = 10000; // 10 seconds cooldown
	const ALREADY_BEST_DISPLAY_TIME = 2000; // 2 seconds to show "Already Best"

	// Check if can submit score (reactive based on conditions)
	$: canSubmitScore = 
		leaderboardId && 
		boardId && 
		$userStore.username && 
		$userStore.passwordHash &&
		$game.data?.board?.player === $userStore.username &&
		!isSubmittingScore &&
		!scoreAlreadyBest &&
		scoreSubmitCooldownRemaining <= 0 &&
		score > 0;

	const handleSubmitScore = async () => {
		if (!boardId || !$userStore.username || !$userStore.passwordHash || !leaderboardId || !$userStore.chainId) return;
		if (isSubmittingScore || scoreAlreadyBest || scoreSubmitCooldownRemaining > 0) return;

		// Check if score would actually trigger a message (score > bestScore from leaderboard)
		if (score <= bestScore) {
			// Score not better than current best - show feedback without sending mutation
			scoreAlreadyBest = true;
			setTimeout(() => {
				scoreAlreadyBest = false;
			}, ALREADY_BEST_DISPLAY_TIME);
			return;
		}

		isSubmittingScore = true;

		try {
			// Must use player's chain client for score submission (not leaderboard chain)
			console.log('🎯 Submit Score Debug:');
			console.log('   userStore.chainId:', $userStore.chainId);
			console.log('   userStore.username:', $userStore.username);
			console.log('   boardId:', boardId);
			console.log('   leaderboardId:', leaderboardId);
			console.log('   current score:', score);
			console.log('   bestScore:', bestScore);
			
			const playerClient = getClient($userStore.chainId);
			const result = submitCurrentScore(
				playerClient,
				boardId,
				$userStore.username,
				$userStore.passwordHash
			);

			if (result) {
				await new Promise<void>((resolve) => {
					result.subscribe((res) => {
						if (res.fetching) return;
						if (res.error) {
							console.warn('❌ Score submission failed:', res.error.message);
						} else {
							console.log('✅ Score submission completed:', res.data);
						}
						resolve();
					});
				});
			}
		} catch (error) {
			console.error('Failed to submit score:', error);
		} finally {
			isSubmittingScore = false;
			// Start cooldown countdown (clear any existing interval first)
			if (scoreSubmitCooldownInterval) {
				clearInterval(scoreSubmitCooldownInterval);
			}
			scoreSubmitCooldownRemaining = Math.ceil(SCORE_SUBMIT_COOLDOWN / 1000);
			scoreSubmitCooldownInterval = setInterval(() => {
				scoreSubmitCooldownRemaining = Math.max(0, scoreSubmitCooldownRemaining - 1);
				if (scoreSubmitCooldownRemaining <= 0 && scoreSubmitCooldownInterval) {
					clearInterval(scoreSubmitCooldownInterval);
					scoreSubmitCooldownInterval = null;
				}
			}, 1000);
		}
	};

	// 🛡️ RESILIENCE: Adaptive polling with backoff
	let initGameIntervalId: ReturnType<typeof setTimeout> | null = null;
	let lastPolledBoardId: string | undefined = undefined;
	let initPollConsecutiveFailures = 0;
	let isInitPollInFlight = false; // 🔒 Prevent overlapping init polls
	const INIT_POLL_MIN_INTERVAL = 500;
	const INIT_POLL_MAX_INTERVAL = 3000; // Max 3 seconds (reduced from 5s for better responsiveness)
	const INIT_POLL_BACKOFF_FACTOR = 1.3; // Gentler backoff (reduced from 1.5)

	// Adaptive interval calculation
	const getAdaptiveInitPollInterval = () => {
		if (initPollConsecutiveFailures === 0) return INIT_POLL_MIN_INTERVAL;
		return Math.min(
			INIT_POLL_MAX_INTERVAL,
			INIT_POLL_MIN_INTERVAL * Math.pow(INIT_POLL_BACKOFF_FACTOR, initPollConsecutiveFailures)
		);
	};

	// Schedule next init poll with adaptive interval
	const scheduleNextInitPoll = () => {
		if (initGameIntervalId) {
			clearTimeout(initGameIntervalId);
			initGameIntervalId = null;
		}
		
		const interval = getAdaptiveInitPollInterval();
		initGameIntervalId = setTimeout(() => {
			doInitPoll();
		}, interval);
	};

	// Actual polling logic
	const doInitPoll = () => {
		// Stop polling if board not found
		if (isBoardNotFound) return;
		
		// 🔒 Skip if already fetching (prevents request pileup during slow responses)
		if ($game.fetching || isInitPollInFlight) {
			// Still schedule next poll, but don't start a new request
			scheduleNextInitPoll();
			return;
		}

		if (boardId && !$game.data?.board) {
			// Board not found yet - count as potential failure
			boardNotFoundCount++;
			initPollConsecutiveFailures++;
			
			// Check if we're still in grace period for newly created boards
			const timeSinceCreation = boardCreationTime ? Date.now() - boardCreationTime : Infinity;
			const isInGracePeriod = timeSinceCreation < NEW_BOARD_GRACE_PERIOD;
			
			// Only trigger "not found" if we've exceeded attempts AND grace period
			if (boardNotFoundCount >= MAX_BOARD_NOT_FOUND_ATTEMPTS && !isInGracePeriod) {
				isBoardNotFound = true;
				// Start redirect countdown (clear any existing interval first)
				if (boardRedirectCountdownInterval) {
					clearInterval(boardRedirectCountdownInterval);
				}
				boardRedirectCountdownInterval = setInterval(() => {
					boardRedirectCountdown--;
					if (boardRedirectCountdown <= 0 && boardRedirectCountdownInterval) {
						clearInterval(boardRedirectCountdownInterval);
						boardRedirectCountdownInterval = null;
						goto('/');
					}
				}, 1000);
				return;
			}
			
			game.reexecute({ requestPolicy: 'network-only' });
			scheduleNextInitPoll();
		} else if ($game.data?.board?.boardId === boardId) {
			// Board found AND matches requested boardId, stop polling
			boardNotFoundCount = 0; // Reset counter
			boardCreationTime = null; // Clear creation time
			initPollConsecutiveFailures = 0; // Reset backoff
			// Don't reschedule - polling complete
		} else if ($game.data?.board?.boardId !== boardId) {
			// Wrong board data (stale), keep polling
			game.reexecute({ requestPolicy: 'network-only' });
			scheduleNextInitPoll();
		}
	};

	$: if (boardId !== lastPolledBoardId) {
		// Clear existing timeout when boardId changes
		if (initGameIntervalId) {
			clearTimeout(initGameIntervalId as unknown as ReturnType<typeof setTimeout>);
			initGameIntervalId = null;
		}

		// Reset not found state when boardId changes
		boardNotFoundCount = 0;
		isBoardNotFound = false;
		boardRedirectCountdown = 5;
		boardCreationTime = Date.now(); // Track when we started looking for this board
		initPollConsecutiveFailures = 0; // Reset backoff

		lastPolledBoardId = boardId;

		// Only start polling if we have a boardId
		if (boardId) {
			// Initial fetch
			game.reexecute({ requestPolicy: 'network-only' });
			// Start adaptive polling
			scheduleNextInitPoll();
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
							}
						})
						.catch(() => {});
				}
			}
		};
	});

	let syncIntervalId: ReturnType<typeof setTimeout> | null = null;
	let syncingBackgroundStartTime: number | null = null;
	
	// 🛡️ RESILIENCE: Adaptive sync polling
	let syncPollConsecutiveFailures = 0;
	const SYNC_POLL_BASE_INTERVAL = 1000; // 1 second base
	const SYNC_POLL_INSPECTOR_INTERVAL = 3000; // 3 seconds for inspector mode
	const SYNC_POLL_MAX_INTERVAL = 10000; // Max 10 seconds
	const SYNC_POLL_BACKOFF_FACTOR = 1.5;
	
	const getAdaptiveSyncInterval = () => {
		const baseInterval = isInspectorMode ? SYNC_POLL_INSPECTOR_INTERVAL : SYNC_POLL_BASE_INTERVAL;
		if (syncPollConsecutiveFailures === 0) return baseInterval;
		return Math.min(
			SYNC_POLL_MAX_INTERVAL,
			baseInterval * Math.pow(SYNC_POLL_BACKOFF_FACTOR, syncPollConsecutiveFailures)
		);
	};
	
	const scheduleNextSyncPoll = () => {
		if (syncIntervalId) {
			clearTimeout(syncIntervalId);
			syncIntervalId = null;
		}
		
		const interval = getAdaptiveSyncInterval();
		syncIntervalId = setTimeout(() => {
			doSyncPoll();
		}, interval);
	};
	
	const doSyncPoll = () => {
		if (offlineMode) {
			scheduleNextSyncPoll();
			return;
		}
		if (!boardId || !$game.data?.board) {
			scheduleNextSyncPoll();
			return;
		}
		
		// 🔧 FIX: Skip sync logic during desync handling (handleDesync manages its own polling)
		if (syncStatus === 'desynced') {
			scheduleNextSyncPoll();
			return;
		}

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
			if (syncStatus === 'syncing-bg' || awaitingBackendSync) {
				if (!syncingBackgroundStartTime) {
					syncingBackgroundStartTime = Date.now();
				}

				if (backendMatchesLocal) {
					// ✅ PERFECT: Backend has caught up completely to our current state
					// 🔒 NOW it's safe to flush - backend has confirmed ALL submitted moves
					
					if (lastSubmittedTimestamp > 0) {
						const flushedCount = flushMovesUpToTimestamp(boardId!, lastSubmittedTimestamp);
						console.log(`✅ Backend confirmed! Flushed ${flushedCount} moves (up to ts: ${lastSubmittedTimestamp})`);
						lastSubmittedTimestamp = 0; // Reset after successful flush
					}
					
					// Update pending count from actual store state
					const remainingMoves = $moveHistoryStore.get(boardId!)?.length || 0;
					pendingMoveCount = remainingMoves;

					awaitingBackendSync = false;
					syncingBackgroundStartTime = null;
					syncStatus = remainingMoves > 0 ? 'pending' : 'ready';
					isFrozen = false;
					lastHashMismatchTime = null;
				} else if (backendInValidSet) {
					// ✅ GOOD: Backend processed our submitted moves (reached a valid state)
					// This means backend processed SOME moves but user made more while syncing
					// 🔒 Flush confirmed moves by timestamp, keep newer ones
					
					if (lastSubmittedTimestamp > 0 && awaitingBackendSync) {
						// Backend is at a valid intermediate state
						// Flush all moves up to the timestamp we submitted
						const flushedCount = flushMovesUpToTimestamp(boardId!, lastSubmittedTimestamp);
						console.log(`✅ Backend at valid state, flushed ${flushedCount} moves (up to ts: ${lastSubmittedTimestamp})`);
						lastSubmittedTimestamp = 0; // Reset after flush
						
						const remainingMoves = $moveHistoryStore.get(boardId!)?.length || 0;
						pendingMoveCount = remainingMoves;
					}
					
					awaitingBackendSync = false;
					syncingBackgroundStartTime = null;
					
					// Transition based on whether we have more pending moves
					const currentRemainingMoves = $moveHistoryStore.get(boardId!)?.length || 0;
					if (currentRemainingMoves > 0) {
						syncStatus = 'pending';
					} else {
						syncStatus = 'synced';
						setTimeout(() => {
							const checkMoves = $moveHistoryStore.get(boardId!)?.length || 0;
							if (checkMoves === 0 && syncStatus === 'synced') {
								syncStatus = 'ready';
							} else if (syncStatus === 'synced') {
								syncStatus = 'pending';
							}
						}, 800);
					}
					isFrozen = false;
					lastHashMismatchTime = null;
				} else {
					// ❌ Backend has unknown state - but we're awaiting sync
					// This can happen when backend processed moves but we don't have all intermediate hashes
					// Trust the backend during active sync - add its hash to valid set
					if (awaitingBackendSync && lastSubmittedTimestamp > 0) {
						// Backend is processing our submission - trust its current state
						console.log(`🔄 Backend at intermediate state during sync, adding to valid set`);
						validBoardHashes.add(backendHash);
						// Don't desync yet - give it more time
					}
					
					const syncWaitTime = Date.now() - syncingBackgroundStartTime;

					if (syncWaitTime > 30000) {
						// Extended timeout (30s) when awaiting sync - server can be slow under load
						console.warn(`⚠️ Sync timeout after ${syncWaitTime}ms - backend at unknown state`);
						syncingBackgroundStartTime = null;
						handleDesync();
					}
				}
			} else {
				// Reset sync timer when not actively syncing
				syncingBackgroundStartTime = null;
			}

			// Continuous hash validation
			if (backendInValidSet) {
				// ✅ Backend state is valid - this is good
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
				// Wait for backend to catch up if:
				// - We have pending moves locally
				// - We have moves submitted but not yet confirmed (lastSubmittedTimestamp > 0)
				// - We're awaiting backend sync
				// - Sync is in progress
				// - We just finished syncing (backend may be processing)
				// - Local game is finished (don't overwrite game over state!)
				// - 🔧 FIX: Recently synced (grace period for slow backend during peak load)
				const SYNC_GRACE_PERIOD = 30000; // 30 seconds grace period after sync (server can be slow under load)
				const recentlySynced = lastSyncTime && (Date.now() - lastSyncTime < SYNC_GRACE_PERIOD);
				
				const shouldWaitForBackend = 
					pendingMoveCount > 0 || 
					lastSubmittedTimestamp > 0 ||  // 🔒 Moves submitted but not yet confirmed by backend
					isSyncInFlight ||  // 🔒 Sync operation in progress
					awaitingBackendSync ||  // Waiting for backend to process
					syncStatus === 'syncing-bg' || 
					syncStatus === 'syncing' ||
					syncStatus === 'synced' ||  // Just finished sync, backend catching up
					recentlySynced ||  // 🔧 FIX: Grace period for slow backend processing
					state?.finished;  // Don't overwrite finished state
				
				if (shouldWaitForBackend) {
					if (lastHashMismatchTime === null) {
						lastHashMismatchTime = Date.now();
					} else if (Date.now() - lastHashMismatchTime > 30000) {
						// 30 seconds of mismatch - trigger desync handler
						handleDesync();
					}
				} else {
					// No pending moves, not syncing, backend has unknown state
					// 🔧 FIX: Double-check moveHistoryStore before resetting
					const storeMovesCount = ($moveHistoryStore.get(boardId!) || []).length;
					if (storeMovesCount > 0) {
						// Still have moves in store - don't reset yet

					} else {
						// Truly no pending moves - safe to reset
						state = createState($game.data.board.board, 4, boardId!, player);
						score = $game.data.board.score || 0;
						
						// Clear and rebuild valid hashes from this new state
						validBoardHashes.clear();
						validBoardHashes.add(backendHash);
						
						// Clear awaiting state since we're resetting
						awaitingBackendSync = false;
						lastSubmittedTimestamp = 0;  // 🔒 Reset submitted timestamp
						
						// Ensure we're in a clean sync state
						syncStatus = 'ready';
						lastHashMismatchTime = null;
					}
				}
			}
		// Schedule next poll with adaptive interval
		scheduleNextSyncPoll();
	};

	onMount(() => {
		// Start adaptive sync polling
		scheduleNextSyncPoll();

		return () => {
			if (syncIntervalId) {
				clearTimeout(syncIntervalId);
				syncIntervalId = null;
			}
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

		const nextMoveIndex = inspectorCurrentMoveIndex + 1;

		// Ensure the next move is loaded
		await loadMoveRange(nextMoveIndex);

		// 🔧 FIX: Check if next move is actually loaded before advancing
		// If still loading (e.g., another load in progress), retry after delay
		const nextMove = paginatedHistoryStore?.getMove(nextMoveIndex);
		if (!nextMove) {
			// Move not loaded yet, retry after a short delay
			inspectorPlayTimeout = setTimeout(() => {
				playNextInspectorMove();
			}, 100);
			return;
		}

		const currentMove = getCurrentMoveData();

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

	// 🚀 Pagination functions for move history (with resilience)
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
			// 🛡️ RESILIENCE: Use requestManager for timeout and circuit breaker protection
			const { requestManager, CircuitOpenError } = await import('$lib/services/requestManager');
			
			const result = await requestManager.request(
				async () => {
					const boardQuery = getBoardPaginated(
						client,
						boardId,
						start - 1, // Convert to 0-based offset
						limit
					);

					// Wait for the query to complete with timeout protection
					return new Promise<{
						data?: { board?: { moveHistory?: MoveHistoryRecord[] } };
					}>((resolve, reject) => {
						let unsubscribe: (() => void) | undefined;
						unsubscribe = boardQuery.subscribe((value) => {
							if (value.error) {
								if (unsubscribe) unsubscribe();
								reject(value.error);
							} else if (value.data !== undefined) {
								if (unsubscribe) unsubscribe();
								resolve(value);
							}
						});
					});
				},
				{
					endpoint: `moves:${boardId}`,
					dedupKey: `moves:${boardId}:${start}:${limit}`,
					timeout: 20000, // 20 second timeout for move loading
					retries: 2,
					priority: 'high'
				}
			);

			if (result?.data?.board?.moveHistory) {
				const moves = result.data.board.moveHistory;
				paginatedHistoryStore.addLoadedRange(start, moves);
				// 🔧 FIX: Update game state after moves are loaded
				handleGameStateUpdate();
			}
		} catch (error: unknown) {
			console.error('Failed to load move range:', error);
			// 🛡️ RESILIENCE: Show user-friendly error for circuit breaker
			if (error && typeof error === 'object' && 'name' in error && error.name === 'CircuitOpenError') {
				console.warn('⚠️ Move loading circuit is open - server may be overloaded');
			}
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
			
			// 🔧 FIX: Wait for move to be loaded before updating state
			// If move not loaded yet (e.g., another load in progress), wait and retry
			let retries = 0;
			while (!paginatedHistoryStore.isMoveLoaded(targetMove) && retries < 50) {
				await new Promise(resolve => setTimeout(resolve, 100));
				retries++;
			}
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
		
		// 🎵 Clean up rhythm engine to stop audio and dispose Tone.js resources
		if (rhythmEngine) {
			rhythmEngine.dispose();
			rhythmEngine = null;
		}
		
		// 🎵 Stop beat phase animation
		stopBeatPhaseAnimation();
		
		// 🧹 Clean up intervals that may still be running
		if (scoreSubmitCooldownInterval) {
			clearInterval(scoreSubmitCooldownInterval);
			scoreSubmitCooldownInterval = null;
		}
		if (boardRedirectCountdownInterval) {
			clearInterval(boardRedirectCountdownInterval);
			boardRedirectCountdownInterval = null;
		}
		
		// 🧹 Clean up memory-holding data structures
		validBoardHashes.clear();
		recentMoves = [];
		moveQueue = [];
		
		// 🧹 Clean up pagination store reference
		paginatedHistoryStore = null;
		initialBoardCache = null;
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

{#if isBoardNotFound}
	<div class="game-container {$boardSize}">
		<div class="flex h-full flex-col items-center justify-center gap-4 p-8 text-center">
			<div class="text-6xl">🎮</div>
			<h2 class="text-xl font-semibold text-gray-200">Game Not Found</h2>
			<p class="text-gray-400">
				This game doesn't exist or hasn't been created yet.
			</p>
			<p class="text-sm text-gray-500">
				Redirecting to home in {boardRedirectCountdown} seconds...
			</p>
			<div class="flex gap-3">
				<button
					type="button"
					class="btn variant-filled-primary"
					onclick={() => {
						boardNotFoundCount = 0;
						isBoardNotFound = false;
						boardRedirectCountdown = 5;
						initPollConsecutiveFailures = 0;
						game.reexecute({ requestPolicy: 'network-only' });
						scheduleNextInitPoll();
					}}
				>
					Retry
				</button>
				<button
					type="button"
					class="btn variant-filled-surface"
					onclick={() => goto('/')}
				>
					Go Home
				</button>
			</div>
		</div>
	</div>
{:else if boardId && !$game.data?.board && !isCreatingNewBoard}
	<!-- 🛡️ Loading state while fetching board -->
	<div class="game-container {$boardSize}">
		<div class="flex h-full flex-col items-center justify-center gap-4 p-8 text-center">
			{#if $game.fetching}
				<div class="h-8 w-8 animate-spin rounded-full border-2 border-purple-400 border-t-transparent"></div>
				<p class="text-gray-400">Loading game...</p>
			{:else if $game.error}
				<div class="text-5xl">😵</div>
				<h2 class="text-xl font-semibold text-red-300">Failed to load game</h2>
				<p class="text-sm text-gray-400">{$game.error.message}</p>
				<button
					type="button"
					class="btn variant-filled-primary"
					onclick={() => {
						initPollConsecutiveFailures = 0;
						game.reexecute({ requestPolicy: 'network-only' });
						scheduleNextInitPoll();
					}}
				>
					Retry
				</button>
			{:else}
				<div class="h-8 w-8 animate-spin rounded-full border-2 border-gray-400 border-t-transparent"></div>
				<p class="text-gray-400">Waiting for board data...</p>
				<p class="text-xs text-gray-500">Attempt {boardNotFoundCount} / {MAX_BOARD_NOT_FOUND_ATTEMPTS}</p>
			{/if}
		</div>
	</div>
{:else}
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
			{beatPhase}
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
							<span class="stat">
								<span class="label">BPM</span>
								<span class="value text-yellow-400">{displayBpm || rhythmEngine?.getBpm() || '?'}</span>
							</span>
						</div>
					{/if}
				</div>
			{/snippet}
		</Board>
		
		<!-- 🎵 Rhythm Start Overlay - requires user interaction to start audio -->
		{#if rhythmNeedsStart && !isInspectorMode}
			<div class="rhythm-start-overlay">
				<div class="rhythm-start-content">
					<div class="rhythm-icon">🎵</div>
					<div class="rhythm-title">Rhythm Mode</div>
					
					<!-- Track info -->
					{#if rhythmSettings?.useMusic}
						<div class="rhythm-track-info">
							{#if rhythmSettings.trackIndex === 'random' || rhythmSettings.trackIndex === undefined}
								<span class="track-label">🎲 Random Track</span>
							{:else}
								{@const trackIdx = typeof rhythmSettings.trackIndex === 'number' ? rhythmSettings.trackIndex : parseInt(rhythmSettings.trackIndex)}
								{@const track = MUSIC_TRACKS[trackIdx]}
								{#if track}
									<span class="track-label">🎵 {track.name}</span>
									<span class="track-bpm">{track.bpm} BPM</span>
								{/if}
							{/if}
						</div>
					{:else}
						<div class="rhythm-track-info">
							<span class="track-label">🔊 Metronome</span>
							<span class="track-bpm">{rhythmSettings?.bpm || 120} BPM</span>
						</div>
					{/if}
					
					<!-- Calibration status -->
					<div class="rhythm-calibration-status">
						{#if hasCalibration}
							<span class="calibrated">✓ Calibrated ({calibrationOffset > 0 ? '+' : ''}{calibrationOffset}ms)</span>
						{:else}
							<span class="not-calibrated">⚠ Not calibrated</span>
						{/if}
					</div>
					
					<!-- Buttons -->
					<div class="rhythm-buttons">
						<button
							class="rhythm-calibrate-btn"
							onclick={openCalibrationModal}
						>
							🎯 Calibrate
						</button>
						<button
							class="rhythm-start-btn"
							onclick={startRhythmEngine}
						>
							▶ Start
						</button>
					</div>
				</div>
			</div>
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
					</button>
				{/if}
			</div>

			<!-- 🚀 Submit Score to Leaderboard Button -->
			{#if leaderboardId && $game.data?.board?.player === $userStore.username && !isInspectorMode}
				<button
					onclick={handleSubmitScore}
					disabled={!canSubmitScore && !scoreAlreadyBest}
					class="flex w-full items-center justify-center gap-2 rounded-lg border px-4 py-2 transition-colors lg:w-auto
						{scoreAlreadyBest
							? 'border-cyan-500/50 bg-cyan-950/50 text-cyan-400'
							: canSubmitScore 
								? 'border-green-500/50 bg-green-950/50 hover:bg-green-900/50 text-green-400' 
								: 'border-green-800/30 bg-green-950/20 text-green-700 cursor-not-allowed opacity-60'}"
					title={scoreAlreadyBest ? 'Score already submitted' : canSubmitScore ? 'Submit current score to leaderboard' : scoreSubmitCooldownRemaining > 0 ? `Wait ${scoreSubmitCooldownRemaining}s` : 'Cannot submit'}
				>
					{#if isSubmittingScore}
						<div class="h-4 w-4 animate-spin rounded-full border-2 border-green-400 border-t-transparent"></div>
						<span class="whitespace-nowrap text-xs lg:text-sm">Submitting...</span>
					{:else if scoreAlreadyBest}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M20 6L9 17l-5-5"/>
						</svg>
						<span class="whitespace-nowrap text-xs lg:text-sm">Already Best</span>
					{:else}
						<svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
							<path d="M12 19V5M5 12l7-7 7 7"/>
						</svg>
						<span class="whitespace-nowrap text-xs lg:text-sm">
							{#if scoreSubmitCooldownRemaining > 0}
								Submit ({scoreSubmitCooldownRemaining}s)
							{:else}
								Submit Score
							{/if}
						</span>
					{/if}
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
		<div class="mt-2 flex flex-wrap items-center justify-between gap-2 sm:gap-3">
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

			<!-- Status - wraps on small screens, inline when space allows -->
			{#if autoPlayEnabled}
				<div class="flex flex-col gap-1 text-xs sm:gap-1.5 sm:text-sm">
					<div class="flex flex-wrap items-center gap-1">
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
					<div class="break-words text-xs text-surface-300 sm:text-sm">
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
{/if}

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
		background: rgba(0, 0, 0, 0.9);
		backdrop-filter: blur(4px);
		z-index: 50;
		border: none;
		border-radius: 8px;
		animation: rhythm-pulse 2s ease-in-out infinite;
	}

	@keyframes rhythm-pulse {
		0%, 100% { box-shadow: 0 0 20px rgba(139, 92, 246, 0.5); }
		50% { box-shadow: 0 0 40px rgba(139, 92, 246, 0.8); }
	}

	.rhythm-start-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.75rem;
		padding: 1.5rem;
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

	.rhythm-track-info {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
		padding: 0.5rem 1rem;
		background: rgba(139, 92, 246, 0.2);
		border-radius: 8px;
		border: 1px solid rgba(139, 92, 246, 0.3);
	}

	.track-label {
		font-size: 0.9rem;
		color: #c4b5fd;
	}

	.track-bpm {
		font-size: 0.8rem;
		color: #9ca3af;
	}

	.rhythm-calibration-status {
		font-size: 0.8rem;
		padding: 0.25rem 0.5rem;
		border-radius: 4px;
	}

	.rhythm-calibration-status .calibrated {
		color: #4ade80;
	}

	.rhythm-calibration-status .not-calibrated {
		color: #fbbf24;
	}

	.rhythm-buttons {
		display: flex;
		gap: 0.75rem;
		margin-top: 0.5rem;
	}

	.rhythm-calibrate-btn {
		padding: 0.75rem 1.25rem;
		background: rgba(107, 114, 128, 0.5);
		border: 1px solid rgba(107, 114, 128, 0.7);
		border-radius: 8px;
		color: white;
		font-weight: 600;
		cursor: pointer;
		transition: all 0.2s;
	}

	.rhythm-calibrate-btn:hover {
		background: rgba(107, 114, 128, 0.7);
	}

	.rhythm-start-btn {
		padding: 0.75rem 1.5rem;
		background: linear-gradient(135deg, #8b5cf6 0%, #a78bfa 100%);
		border: none;
		border-radius: 8px;
		color: white;
		font-weight: 600;
		font-size: 1.1rem;
		cursor: pointer;
		transition: all 0.2s;
	}

	.rhythm-start-btn:hover {
		transform: scale(1.05);
		box-shadow: 0 0 20px rgba(139, 92, 246, 0.5);
	}

	@media (max-width: 640px) {
		.rhythm-icon {
			font-size: 2.5rem;
		}

		.rhythm-title {
			font-size: 1.25rem;
		}

		.rhythm-buttons {
			flex-direction: column;
			width: 100%;
		}

		.rhythm-calibrate-btn,
		.rhythm-start-btn {
			width: 100%;
		}
	}
</style>
