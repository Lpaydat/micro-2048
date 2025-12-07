<script lang="ts">
	import { onMount } from 'svelte';
	import { userStore } from '$lib/stores/userStore';
	import RefreshCw from 'lucide-svelte/icons/refresh-cw';
	import LeaderboardDetails from '../molecules/LeaderboardDetails.svelte';
	import ListItem from '../molecules/LeaderboardItem.svelte';
	import LeaderboardRankers from '../molecules/LeaderboardRankers.svelte';
	import { getClient } from '$lib/client';
	import { requestLeaderboardRefresh } from '$lib/graphql/mutations/requestLeaderboardRefresh';

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
	}

	let {
		isFullScreen,
		leaderboardId,
		rankers = [],
		endCallback,
		forceAllEnded = false,
		currentScore = 0,
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

	// Show refresh button only when current score > leaderboard score
	const shouldShowRefreshButton = $derived.by(() => {
		if (!$userStore.username || !leaderboardId) return false;
		return currentScore > playerLeaderboardScore;
	});

	// Leaderboard refresh state
	const playerClient = $derived(getClient($userStore.chainId, true));
	let lastRefreshTime = $state(0);
	let isRefreshing = $state(false);
	let now = $state(Date.now()); // Reactive time for countdown
	const REFRESH_COOLDOWN_MS = 10000; // 10 seconds cooldown

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
			const result = requestLeaderboardRefresh(playerClient, leaderboardId);
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
