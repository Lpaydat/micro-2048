<script lang="ts">
	import BlockHashes from '../molecules/BlockHashes.svelte';
	import Game from '../organisms/Game.svelte';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import { userStore } from '$lib/stores/userStore';
	import { isHashesListVisible } from '$lib/stores/hashesStore';
	import Brand from '../molecules/Brand.svelte';
	import SideLeaderboard from '../organisms/SideLeaderboard.svelte';
	import MobileRankerStats from '../organisms/MobileRankerStats.svelte';
	import { getLeaderboardDetails } from '$lib/graphql/queries/getLeaderboardDetails';
	import { onDestroy, onMount } from 'svelte';
	import LeaderboardDetails from '../molecules/LeaderboardDetails.svelte';
	import { page } from '$app/stores';
	import { getClient } from '$lib/client';

	let boardId = $state<string>($page.url.searchParams.get('boardId') ?? '');
	let leaderboardId = $state<string | undefined>(
		$page.url.searchParams.get('leaderboardId') ?? undefined
	);
	let chainId = $derived(boardId.split('.')[0] ?? $userStore.chainId);

	let unsubscribe: any;

	$effect(() => {
		unsubscribe = page.subscribe(($page) => {
			boardId = $page.url.searchParams.get('boardId') ?? '';
			leaderboardId = $page.url.searchParams.get('leaderboardId') ?? undefined;
		});
	});

	onDestroy(() => {
		if (unsubscribe) unsubscribe();
	});
	let currentPlayerScore = $state<number>(0);
	let bestScore = $state<number>(0);
	let isEnded = $state(false);

	const overlayMessage = $derived(leaderboardId ? 'Game Over!' : undefined);
	const canStartNewGame = $derived(!!$userStore.username && !isEnded);
	const canMakeMove = $derived.by(() => {
		if (!$userStore.username || !canStartNewGame) return false;

		const endTime = Number($leaderboard?.data?.leaderboard?.endTime ?? '0');
		if (endTime > 0 && endTime <= Date.now()) {
			return false;
		}

		return true;
	});

	const endCallback = () => {
		isEnded = true;
	};

	const client = $derived(getClient(leaderboardId, true));
	const leaderboard = $derived(getLeaderboardDetails(client));

	const rankers = $derived(
		$leaderboard?.data?.leaderboard?.rankers?.sort(
			(a: { score: number }, b: { score: number }) => b.score - a.score
		) ?? []
	);
	
	// Get tournament description for rhythm mode detection
	const tournamentDescription = $derived($leaderboard?.data?.leaderboard?.description ?? '');

	$effect(() => {
		bestScore =
			rankers?.find(
				(ranker: { username: string; score: number }) => ranker.username === $userStore.username
			)?.score ?? 0;
	});

	let intervalId: NodeJS.Timeout;
	onMount(() => {
		intervalId = setInterval(() => {
			leaderboard?.reexecute({ requestPolicy: 'network-only' });
		}, 1000);

		return () => clearInterval(intervalId);
	});
</script>

<MainTemplate mainCenter>
	{#snippet header()}
		<MobileRankerStats
			player={$userStore.username ?? ''}
			{rankers}
			currentScore={currentPlayerScore}
			{leaderboardId}
		/>
	{/snippet}

	{#snippet subHeader()}
		{#if leaderboardId}
			<div class="mx-4 mt-4 w-fit flex-none md:mt-6">
				<LeaderboardDetails
					{...$leaderboard?.data?.leaderboard}
					showName
					{leaderboardId}
					{endCallback}
				/>
			</div>
		{/if}
	{/snippet}

	{#snippet sidebar()}
		{#if $userStore.username}
			<Brand />
			<SideLeaderboard
				{...$leaderboard?.data?.leaderboard}
				showName
				{rankers}
				{leaderboardId}
				{endCallback}
				currentScore={currentPlayerScore}
			/>
		{:else}
			<UserSidebar />
		{/if}
	{/snippet}

	{#snippet main()}
		<div class="flex h-full items-center justify-center">
			<div class="my-auto w-full max-w-2xl">
				<Game
					bind:leaderboardId
					bind:score={currentPlayerScore}
					player={$userStore.username ?? ''}
					{boardId}
					{chainId}
					{canStartNewGame}
					showBestScore
					{isEnded}
					{canMakeMove}
					{bestScore}
					{overlayMessage}
					{tournamentDescription}
				/>
			</div>
		</div>
		{#if $isHashesListVisible}
			<BlockHashes />
		{/if}
	{/snippet}
</MainTemplate>
