<script lang="ts">
	import { onMount } from 'svelte';
	import { userStore } from '$lib/stores/userStore';
	import RefreshCw from 'lucide-svelte/icons/refresh-cw';
	import LeaderboardDetails from '../molecules/LeaderboardDetails.svelte';
	import ListItem from '../molecules/LeaderboardItem.svelte';
	import LeaderboardRankers from '../molecules/LeaderboardRankers.svelte';
	import { getClient } from '$lib/client';
	import { requestLeaderboardRefresh } from '$lib/graphql/mutations/requestLeaderboardRefresh';
	import { submitCurrentScore } from '$lib/graphql/mutations/submitCurrentScore';
	import { getBoardId } from '$lib/stores/boardId';

	interface Props {
		isFullScreen?: boolean;
		leaderboardId?: string;
		name?: string;
		host?: string;
		startTime?: string;
		endTime?: string;
		totalBoards?: number;
		totalPlayers?: number;
		rankers?: { username: string; score: number; boardId: string; isEnded?: boolean }[];
		endCallback?: () => void;
		forceAllEnded?: boolean; // TEST: Force all boards to show as ended
		currentScore?: number; // Current board score for refresh button visibility
		lastUpdate?: string; // Backend leaderboard last update timestamp (milliseconds)
	}

	let {
		isFullScreen,
		leaderboardId,
		rankers = [],
		endCallback,
		forceAllEnded = false,
		currentScore = 0,
		lastUpdate,
		...rest
	}: Props = $props();

	const player = $derived($userStore.username);
	const customClass = isFullScreen ? 'w-full h-full' : 'p-6 w-80 max-h-full max-w-md mx-auto';
	const containerClass = isFullScreen ? 'mt-4' : 'mt-6';

	const getBoardUrl = (boardId: string) => {
		return leaderboardId
			? `/game/?boardId=${boardId}&leaderboardId=${leaderboardId}`
			: `/game/?boardId=${boardId}`;
	};

	// Get current player's best score from leaderboard
	const playerLeaderboardScore = $derived.by(() => {
		if (!player || !rankers?.length) return 0;
		const playerRanker = rankers.find((r) => r.username === player);
		return playerRanker?.score ?? 0;
	});
	
	// Get leaderboard last update time (from backend, in milliseconds)
	const leaderboardLastUpdate = $derived(lastUpdate ? Number(lastUpdate) : 0);

	// Show refresh button when:
	// 1. Current score > leaderboard score (player has higher score to submit)
	// 2. OR leaderboard is stale (30+ seconds since backend last update)
	const shouldShowRefreshButton = $derived.by(() => {
		if (!$userStore.username || !leaderboardId) return false;
		
		// Check if player has higher score than leaderboard
		const hasHigherScore = currentScore > playerLeaderboardScore;
		
		// Check if leaderboard is stale (backend timestamp > 30s old)
		const isBackendStale = 
			leaderboardLastUpdate > 0 && 
			(now - leaderboardLastUpdate) >= 30000; // 30 seconds
		
		return hasHigherScore || isBackendStale;
	});

	// Leaderboard refresh state
	// 🚀 IMPROVED: Call mutation directly on leaderboard chain (leaderboardId IS the chain ID)
	// Shared 10s cooldown with backend - manual refresh and auto-triggers share this
	const leaderboardClient = $derived(leaderboardId ? getClient(leaderboardId, true) : null);
	// Player client for submitting score (must use player's chain, not main chain)
	const playerClient = $derived($userStore.chainId ? getClient($userStore.chainId, false) : null);
	const currentBoardId = $derived(leaderboardId ? getBoardId(leaderboardId) : null);
	
	let lastRefreshTime = $state(0);
	let isRefreshing = $state(false);
	let now = $state(Date.now()); // Reactive time for countdown
	const REFRESH_COOLDOWN_MS = 15000; // 15 seconds cooldown (matches backend)
	
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

	const canRefresh = $derived.by(() => {
		if (!shouldShowRefreshButton) return false;
		return now - lastRefreshTime >= REFRESH_COOLDOWN_MS;
	});

	const refreshCooldownRemaining = $derived.by(() => {
		const remaining = REFRESH_COOLDOWN_MS - (now - lastRefreshTime);
		return remaining > 0 ? Math.ceil(remaining / 1000) : 0;
	});

	const triggerLeaderboardRefresh = async () => {
		if (!canRefresh || isRefreshing || !leaderboardClient || !leaderboardId) return;

		isRefreshing = true;
		lastRefreshTime = Date.now();

		try {
			// 🚀 Step 1: Submit current score from player's board (if applicable)
			// Only send if score > lastSubmittedScore (avoid unnecessary mutations)
			if (currentBoardId && $userStore.username && $userStore.passwordHash && $userStore.chainId && playerClient) {
				const lastSubmitted = getLastSubmittedScore(leaderboardId);
				
				if (currentScore > lastSubmitted) {
					console.log('✅ Submitting current score before refresh...', { boardId: currentBoardId, currentScore, lastSubmitted });
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
									console.log('✅ Score submitted successfully');
									setLastSubmittedScore(leaderboardId!, currentScore);
								}
								resolve();
							});
						});
					}
					// Wait for message to propagate
					await new Promise(resolve => setTimeout(resolve, 1500));
				} else {
					console.log('⏭️ Score not better than last submitted, skipping', { currentScore, lastSubmitted });
				}
			}

			// 🚀 Step 2: Call updateLeaderboard mutation directly on leaderboard chain
			const result = requestLeaderboardRefresh(leaderboardClient);
			if (result) {
				result.subscribe((res) => {
					if (res.error) {
						console.error('Leaderboard refresh failed:', res.error);
					} else {
						console.log('Leaderboard refresh triggered successfully');
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
</script>

<div class="mx-auto flex max-w-sm flex-col gap-4 {containerClass} overflow-hidden" style="max-height: calc(100vh - 2rem);">
	{#if leaderboardId}
		<div class="me-auto flex flex-none">
			<LeaderboardDetails {leaderboardId} {...rest} {endCallback} />
		</div>
	{/if}
	<div
		class="text-center {customClass} flex min-h-0 flex-1 flex-col overflow-hidden rounded-lg bg-[#FAF8EF] shadow-md"
	>
		<header class="mb-4 flex flex-none flex-col items-center pt-4">
			<div class="mb-2 flex items-center gap-2">
				<h1 class="text-2xl font-bold text-[#776E65]">Leaderboard</h1>
				{#if shouldShowRefreshButton}
					<button
						type="button"
						class="relative rounded-full p-1 transition-colors hover:bg-[#EDE0C8]"
						onclick={triggerLeaderboardRefresh}
						disabled={!canRefresh || isRefreshing}
						title={canRefresh ? 'Refresh leaderboard scores' : `Wait ${refreshCooldownRemaining}s`}
					>
						<RefreshCw
							size={18}
							class={isRefreshing ? 'animate-spin' : ''}
							strokeWidth={2}
							color={canRefresh ? '#4ade80' : '#9ca3af'}
						/>
						{#if !canRefresh && refreshCooldownRemaining > 0}
							<span
								class="absolute -bottom-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full bg-[#776E65] text-[10px] text-white"
							>
								{refreshCooldownRemaining}
							</span>
						{/if}
					</button>
				{/if}
			</div>
			<p class="text-xs text-[#8F7A66]/60">Tap any player to watch their game</p>
		</header>

		<div class="min-h-0 flex-1 overflow-hidden">
			<LeaderboardRankers {rankers}>
				{#snippet item(rank, username, score, boardId, isEliminated, isEnded)}
					<ListItem
						{rank}
						name={username}
						isCurrentPlayer={username === player}
						{score}
						{boardId}
						boardUrl={getBoardUrl(boardId)}
						isEnded={forceAllEnded ? true : isEnded}
					/>
				{/snippet}
			</LeaderboardRankers>
		</div>
	</div>
</div>

<style>
	div {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
	}

	h1 {
		text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.1);
	}

	.border-sm {
		border-radius: 6px !important;
	}
</style>
