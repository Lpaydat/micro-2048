<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import Trash from 'lucide-svelte/icons/trash-2';
	import Star from 'lucide-svelte/icons/star';
	import Plus from 'lucide-svelte/icons/plus';
	import GitBranchPlus from 'lucide-svelte/icons/git-branch-plus';
	import ContinueIcon from 'lucide-svelte/icons/step-forward';
	import RefreshCw from 'lucide-svelte/icons/refresh-cw';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import RankerLeaderboard from '../organisms/RankerLeaderboard.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import PageHeader from '../molecules/PageHeader.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import { userStore } from '$lib/stores/userStore';
	import { goto } from '$app/navigation';
	import { getBoardId, setBoardId } from '$lib/stores/boardId';
	import {
		deleteLeaderboard,
		togglePinLeaderboard
	} from '$lib/graphql/mutations/leaderboardAction.ts';
	import { getModalStore, type ModalSettings } from '@skeletonlabs/skeleton';
	import { getClient } from '$lib/client';
	import { newGameBoard } from '$lib/game/newGameBoard';
	import { newShard } from '$lib/graphql/mutations/newShard';
	import { addShards, getShards } from '$lib/stores/shards';
	import { getBoard } from '$lib/graphql/queries/getBoard';
	import { requestFaucetMutation } from '$lib/graphql/mutations/requestFaucet';
	import { requestLeaderboardRefresh } from '$lib/graphql/mutations/requestLeaderboardRefresh';
	import { submitCurrentScore } from '$lib/graphql/mutations/submitCurrentScore';

	interface Props {
		leaderboardId?: string;
		prevPage?: string;
		updateInterval?: number;
	}

	let { leaderboardId = '', prevPage, updateInterval = 5000 }: Props = $props();

	const LEADERBOARD = gql`
		query Leaderboard {
			leaderboard {
				leaderboardId
				name
				description
				host
				startTime
				endTime
				totalBoards
				totalPlayers
				isPinned
				lastUpdate
				rankers {
					username
					score
					boardId
					isEnded
				}
				shardIds
			}
			balance
		}
	`;

	const mainClient = getContextClient();
	// Must be $derived so it updates when leaderboardId changes (navigation between tournaments)
	const leaderboardClient = $derived(getClient(leaderboardId, true));
	// Player client should only be created when we have a valid player chain ID
	// Do NOT use main chain as default - that would send player mutations to wrong chain!
	const playerClient = $derived($userStore.chainId ? getClient($userStore.chainId, false) : null);

	const leaderboard = $derived(
		queryStore({
			client: leaderboardClient,
			query: LEADERBOARD
		})
	);
	const currentBoardId = $derived(getBoardId(leaderboardId));
	const board = $derived(playerClient ? getBoard(playerClient) : null);

	const isEnded = $derived.by(() => {
		const endTime = Number($leaderboard?.data?.leaderboard?.endTime ?? '0');
		if (endTime === 0) return false;
		return endTime - Date.now() < 0;
	});
	const isStarted = $derived.by(() => {
		const startTime = Number($leaderboard?.data?.leaderboard?.startTime ?? '0');
		if (startTime === 0) return true;
		return startTime - Date.now() < 0;
	});
	const canDeleteEvent = $derived(
		($leaderboard?.data?.leaderboard?.host === $userStore.username || $userStore.isMod) && !isEnded
	);
	const balance = $derived($leaderboard.data?.balance);
	const canPinEvent = $derived($userStore.isMod && !isEnded);
	const canPlayGame = $derived(isStarted && !isEnded && $userStore.username);
	const isPinned = $derived($leaderboard?.data?.leaderboard?.isPinned);

	let newGameAt = $state(Date.now());
	let isNewGameCreated = $state(false);
	let isCreatingNewGame = $state(false);

	// Leaderboard refresh state
	let lastRefreshTime = $state(0);
	let isRefreshing = $state(false);
	let now = $state(Date.now()); // Reactive time for countdown
	const REFRESH_COOLDOWN_MS = 15000; // 15 seconds cooldown
	
	// Track last submitted score per tournament (localStorage) to avoid unnecessary mutations
	const getLastSubmittedScore = (tournamentId: string): number => {
		if (typeof window === 'undefined') return 0;
		const key = `lastSubmittedScore-${tournamentId}`;
		return parseInt(localStorage.getItem(key) || '0', 10);
	};

	const setLastSubmittedScore = (tournamentId: string, score: number) => {
		if (typeof window === 'undefined') return;
		const key = `lastSubmittedScore-${tournamentId}`;
		localStorage.setItem(key, score.toString());
	};

	// Not found tracking - stop polling after consecutive failures
	let notFoundCount = $state(0);
	let isNotFound = $state(false);
	let redirectCountdown = $state(5);
	let redirectCountdownInterval: ReturnType<typeof setInterval> | null = null; // Track interval for cleanup
	let leaderboardLoadStartTime: number | null = $state(null); // Track when we started looking
	const MAX_NOT_FOUND_ATTEMPTS = 10; // Stop after 10 consecutive not found (~50 seconds at 5s interval)
	const NEW_LEADERBOARD_GRACE_PERIOD = 30000; // 30 seconds grace period for newly created tournaments

	// Get current player's best score from leaderboard
	const playerLeaderboardScore = $derived.by(() => {
		const username = $userStore.username;
		const rankers = $leaderboard?.data?.leaderboard?.rankers;
		if (!username || !rankers?.length) return 0;
		const playerRanker = rankers.find((r: { username: string }) => r.username === username);
		return playerRanker?.score ?? 0;
	});

	// Get current board score (from player's active board)
	const currentBoardScore = $derived($board?.data?.board?.score ?? 0);
	
	// Get current board owner
	const currentBoardPlayer = $derived($board?.data?.board?.player);
	
	// Get leaderboard last update time (from backend, in milliseconds)
	const leaderboardLastUpdate = $derived(
		Number($leaderboard?.data?.leaderboard?.lastUpdate ?? 0)
	);

	// Show refresh button when:
	// 1. Current board belongs to logged-in user AND current score > leaderboard score
	// 2. OR leaderboard is stale (30+ seconds since backend last update)
	const shouldShowRefreshButton = $derived.by(() => {
		if (!$userStore.username || !leaderboardId) return false;
		
		// Check if viewing own board with higher score
		const isOwnBoardWithHigherScore = 
			currentBoardPlayer === $userStore.username && 
			currentBoardScore > playerLeaderboardScore;
		
		// Check if leaderboard is stale (backend timestamp > 30s old)
		const isBackendStale = 
			leaderboardLastUpdate > 0 && 
			(now - leaderboardLastUpdate) >= 30000; // 30 seconds
		
		return isOwnBoardWithHigherScore || isBackendStale;
	});

	const canRefresh = $derived.by(() => {
		if (!shouldShowRefreshButton) return false;
		return now - lastRefreshTime >= REFRESH_COOLDOWN_MS;
	});

	const refreshCooldownRemaining = $derived.by(() => {
		const remaining = REFRESH_COOLDOWN_MS - (now - lastRefreshTime);
		return remaining > 0 ? Math.ceil(remaining / 1000) : 0;
	});

	const triggerLeaderboardRefresh = async () => {
		if (!canRefresh || isRefreshing || !leaderboardId) return;

		isRefreshing = true;
		lastRefreshTime = Date.now();

		try {
			// 🚀 Step 1: Submit current score from player's board (if applicable)
			// Only send if score > lastSubmittedScore (avoid unnecessary mutations)
			if (currentBoardId && $userStore.username && $userStore.passwordHash && $userStore.chainId && playerClient) {
				const lastSubmitted = getLastSubmittedScore(leaderboardId);
				
				if (currentBoardScore > lastSubmitted) {
					const scoreResult = submitCurrentScore(
						playerClient,
						currentBoardId,
						$userStore.username,
						$userStore.passwordHash
					);
					if (scoreResult) {
						await new Promise<void>((resolve) => {
							scoreResult.subscribe((res) => {
								if (res.fetching) return;
								if (res.error) {
									console.warn('❌ Score submission failed:', res.error.message);
								} else {
									setLastSubmittedScore(leaderboardId!, currentBoardScore);
								}
								resolve();
							});
						});
					}
					// Wait for message to propagate
					await new Promise(resolve => setTimeout(resolve, 1500));
				} else {
				}
			}

			// 🚀 Step 2: Call updateLeaderboard mutation directly on leaderboard chain
			// This triggers block production which processes pending SubmitScore messages
			const result = requestLeaderboardRefresh(leaderboardClient);
			if (result) {
				result.subscribe((res) => {
					if (res.error) {
						console.error('Leaderboard refresh failed:', res.error);
					} else {
						// Refresh the leaderboard data after a short delay
						setTimeout(() => {
							leaderboard.reexecute({ requestPolicy: 'network-only' });
						}, 2000);
					}
					isRefreshing = false;
				});
			} else {
				isRefreshing = false;
			}
		} catch (error) {
			console.error('Failed to trigger leaderboard refresh:', error);
			isRefreshing = false;
		}
	};

	// Update countdown timer every second
	onMount(() => {
		const interval = setInterval(() => {
			now = Date.now();
		}, 1000);
		return () => clearInterval(interval);
	});

	const newEventGame = async () => {
		if (isNewGameCreated || isCreatingNewGame) return;
		if (!leaderboardId || !$userStore.username) return;

		isCreatingNewGame = true;
		newGameAt = Date.now();
		isNewGameCreated = true;

		try {
			await newGameBoard(leaderboardId, newGameAt.toString());
		} catch (error) {
			console.error('Failed to create new game:', error);
			isCreatingNewGame = false;
			isNewGameCreated = false;
		}
	};

	const deleteEventGame = () => {
		deleteLeaderboard(mainClient, leaderboardId);
		setTimeout(() => {
			goto('/events');
		}, 250);
	};

	const togglePin = () => {
		togglePinLeaderboard(mainClient, leaderboardId);
		setTimeout(() => {
			leaderboard.reexecute({ requestPolicy: 'network-only' });
		}, 1000);
	};

	let isCreatingShards = $state(false);
	const createShards = () => {
		if (
			leaderboardId &&
			!isCreatingShards &&
			!$leaderboard.fetching &&
			!$leaderboard.data?.leaderboard?.shardIds?.length
		) {
			isCreatingShards = true;
			Array.from({ length: 16 }).forEach(() => {
				newShard(leaderboardClient);
			});
		}
	};

	const faucet = () => {
		requestFaucetMutation(leaderboardClient);
	};

	const modalStore = getModalStore();
	const modal: ModalSettings = {
		type: 'confirm',
		title: 'Delete Event',
		body: 'Are you sure you want to delete this event?',
		response: (confirmed) => {
			if (confirmed) {
				deleteEventGame();
			}
		}
	};

	let size = $state('md');
	const updateSize = () => {
		if (window.innerWidth < 480) size = 'sm';
		else if (window.innerWidth < 1440) size = 'md';
		else size = 'lg';
	};

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
				// Check if board data is valid and ready
				const boardData = $board?.data?.board;
				const hasValidBoard =
					boardData?.board && Array.isArray(boardData?.board) && boardData?.board.length > 0;

				// Reset states
				isCreatingNewGame = false;
				newGameAt = 0;

				const url = new URL('/game', window.location.origin);
				url.searchParams.set('boardId', $board?.data?.board?.boardId);
				url.searchParams.set('leaderboardId', leaderboardId);
				
				// Pass tournament description for rhythm mode detection on first load
				// This is needed because leaderboard chain may not have data yet
				const description = $leaderboard?.data?.leaderboard?.description;
				if (description) {
					url.searchParams.set('description', description);
				}

				setBoardId($board.data?.board?.boardId, leaderboardId);

				// Conditional reload: force reload if board data isn't valid
				if (hasValidBoard) {
					// Normal redirect - board data is ready
					goto(url.toString(), { replaceState: false });
				} else {
					// Force reload to ensure fresh data fetch
					goto(url.toString(), { replaceState: false, invalidateAll: true });
				}
			}
		}, 1000);

		return () => clearInterval(interval);
	});

	$effect(() => {
		if ($leaderboard.data?.leaderboard?.shardIds?.length) {
			const shards = getShards(leaderboardId);
			if (!shards?.length) {
				addShards(leaderboardId, $leaderboard.data?.leaderboard?.shardIds);
			}
		}
	});

	onMount(() => {
		// Track when we started looking for this leaderboard
		leaderboardLoadStartTime = Date.now();
		leaderboard.reexecute({ requestPolicy: 'network-only' });

		const interval = setInterval(() => {
			// Stop polling if not found
			if (isNotFound) return;

			// Check if leaderboard data exists - only count when NOT fetching
			if (!$leaderboard.fetching) {
				if ($leaderboard.data?.leaderboard?.leaderboardId) {
					// Found - reset counter
					notFoundCount = 0;
					leaderboardLoadStartTime = null;
				} else {
					// Not found - increment counter
					notFoundCount++;
					
					// Check if we're still in grace period for newly created tournaments
					const timeSinceStart = leaderboardLoadStartTime ? Date.now() - leaderboardLoadStartTime : Infinity;
					const isInGracePeriod = timeSinceStart < NEW_LEADERBOARD_GRACE_PERIOD;
					
					// Only trigger "not found" if we've exceeded attempts AND grace period
					if (notFoundCount >= MAX_NOT_FOUND_ATTEMPTS && !isInGracePeriod) {
						isNotFound = true;
						// Start redirect countdown (clear any existing interval first)
						if (redirectCountdownInterval) {
							clearInterval(redirectCountdownInterval);
						}
						redirectCountdownInterval = setInterval(() => {
							redirectCountdown--;
							if (redirectCountdown <= 0 && redirectCountdownInterval) {
								clearInterval(redirectCountdownInterval);
								redirectCountdownInterval = null;
								goto('/');
							}
						}, 1000);
						return;
					}
				}
			}

			leaderboard.reexecute({ requestPolicy: 'network-only' });
		}, updateInterval);

		return () => clearInterval(interval);
	});

	onMount(() => {
		updateSize();
		window.addEventListener('resize', updateSize);
		return () => window.removeEventListener('resize', updateSize);
	});

	onDestroy(() => {
		// Clean up redirect countdown interval if still running
		if (redirectCountdownInterval) {
			clearInterval(redirectCountdownInterval);
			redirectCountdownInterval = null;
		}
	});
</script>

<MainTemplate>
	{#snippet sidebar()}
		<UserSidebar />
	{/snippet}

	{#snippet main()}
		{#if isNotFound}
			<div class="flex h-full flex-col items-center justify-center gap-4 p-8 text-center">
				<div class="text-6xl">🔍</div>
				<h2 class="text-xl font-semibold text-gray-200">Tournament Not Found</h2>
				<p class="text-gray-400">
					This tournament doesn't exist or hasn't been created yet.
				</p>
				<p class="text-sm text-gray-500">
					Redirecting to home in {redirectCountdown} seconds...
				</p>
				<div class="flex gap-3">
					<button
						type="button"
						class="btn variant-filled-primary"
						onclick={() => {
							notFoundCount = 0;
							isNotFound = false;
							redirectCountdown = 5;
							leaderboard.reexecute({ requestPolicy: 'network-only' });
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
		{:else if !leaderboardId}
			<RankerLeaderboard
				rankers={$leaderboard.data?.leaderboard?.rankers}
				{balance}
				{...$leaderboard.data?.leaderboard}
			/>
		{:else}
			<PageHeader title={$leaderboard?.data?.leaderboard?.name} {prevPage}>
				{#snippet subActions()}
					{#if shouldShowRefreshButton}
						<button
							type="button"
							class="btn-icon relative"
							onclick={triggerLeaderboardRefresh}
							disabled={!canRefresh || isRefreshing}
							title={canRefresh ? 'Refresh leaderboard scores' : `Wait ${refreshCooldownRemaining}s`}
						>
							<RefreshCw
								size={20}
								class={isRefreshing ? 'animate-spin' : ''}
								strokeWidth={2}
								color={canRefresh ? '#4ade80' : '#6b7280'}
							/>
							{#if !canRefresh && refreshCooldownRemaining > 0}
								<span
									class="absolute -bottom-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-gray-700 text-[10px] text-gray-300"
								>
									{refreshCooldownRemaining}
								</span>
							{/if}
						</button>
					{/if}
					{#if canPinEvent}
						<button type="button" class="btn-icon" onclick={togglePin}>
							<Star size={20} fill={isPinned ? '#FFCC00' : 'none'} strokeWidth={isPinned ? 0 : 2} />
						</button>
					{/if}
					{#if canDeleteEvent}
						<button type="button" class="btn-icon" onclick={() => modalStore.trigger(modal)}>
							<Trash size={20} />
						</button>
					{/if}
					{#if $userStore.isMod}
						<button type="button" class="btn-icon" onclick={faucet}>
							<Plus size={20} />
						</button>
						<button type="button" class="btn-icon" onclick={createShards}>
							<GitBranchPlus size={20} />
						</button>
					{/if}
				{/snippet}
				{#snippet actions()}
					{#if canPlayGame}
						{#if currentBoardId}
							<a href={`/game/?boardId=${currentBoardId}&leaderboardId=${leaderboardId}`}>
								<ActionButton label="RESUME GAME" onlyIcon={size === 'sm'}>
									{#snippet icon()}
										{#if size === 'sm'}
											<ContinueIcon size={16} />
										{/if}
									{/snippet}
								</ActionButton>
							</a>
						{/if}
						<ActionButton
							label={isCreatingNewGame ? 'CREATING...' : 'NEW GAME'}
							onclick={newEventGame}
							disabled={isCreatingNewGame || isNewGameCreated}
						/>
					{/if}
				{/snippet}
			</PageHeader>
			<RankerLeaderboard
				rankers={$leaderboard.data?.leaderboard?.rankers}
				{balance}
				hasSubHeader
				{...$leaderboard.data?.leaderboard}
				{leaderboardId}
			/>
		{/if}
	{/snippet}
</MainTemplate>
