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
	import { getContextClient } from '@urql/svelte';
	import { getLeaderboardDetails } from '$lib/graphql/queries/getLeaderboardDetails';
	import { onDestroy, onMount } from 'svelte';
	import LeaderboardDetails from '../molecules/LeaderboardDetails.svelte';
	import { page } from '$app/stores';
	import { getClient } from '$lib/client';
	import { applicationId, port } from '$lib/constants';

	let boardId = $state<string>($page.url.searchParams.get('boardId') ?? '');

	let unsubscribe: any;

	$effect(() => {
		unsubscribe = page.subscribe(($page) => {
			boardId = $page.url.searchParams.get('boardId') ?? '';
		});
	});

	onDestroy(() => {
		if (unsubscribe) unsubscribe();
	});

	let leaderboardId = $state<string | undefined>();
	let currentPlayerScore = $state<number>(0);
	let bestScore = $state<number>(0);
	let isEnded = $state(false);

	const overlayMessage = $derived(leaderboardId ? 'Game Over!' : undefined);
	const canStartNewGame = $derived(!!$userStore.username && !isEnded);
	const canMakeMove = $derived(!!$userStore.username && canStartNewGame);

	const endCallback = () => {
		isEnded = true;
	};

	// TODO: use leaderboard chainId
	$effect(() => {
		console.log('leaderboardId', leaderboardId);
	});
	const client = $derived(getClient(leaderboardId, applicationId, port));
	const leaderboard = $derived(
		leaderboardId !== undefined && client !== undefined
			? getLeaderboardDetails(client, leaderboardId)
			: undefined
	);

	const rankers = $derived(
		$leaderboard?.data?.leaderboard?.rankers?.sort(
			(a: { score: number }, b: { score: number }) => b.score - a.score
		) ?? []
	);

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
		<MobileRankerStats player={$userStore.username ?? ''} {rankers} />
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
					{canStartNewGame}
					showBestScore
					{isEnded}
					{canMakeMove}
					{bestScore}
					{overlayMessage}
				/>
			</div>
		</div>
		{#if $isHashesListVisible}
			<BlockHashes />
		{/if}
	{/snippet}
</MainTemplate>
