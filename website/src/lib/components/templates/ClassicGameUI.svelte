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
	import { onMount } from 'svelte';

	const canMakeMove = $derived(!!$userStore.username);
	const chainId = $derived($userStore.chainId);

	let leaderboardId = $state<string>('');
	let currentPlayerScore = $state<number>(0);

	const client = getContextClient();
	const leaderboard = $derived(getLeaderboardDetails(client, leaderboardId));

	const rankers = $derived(
		$leaderboard?.data?.leaderboard.rankers
			.sort((a: { score: number }, b: { score: number }) => b.score - a.score)
			.map((ranker: { username: string; score: number }, index: number) => {
				const score = ranker.username === $userStore.username ? currentPlayerScore : ranker.score;
				return { ...ranker, score, rank: index + 1 };
			}) ?? []
	);

	let intervalId: NodeJS.Timeout;
	onMount(() => {
		intervalId = setInterval(() => {
			leaderboard.reexecute({ requestPolicy: 'network-only' });
		}, 1000);

		return () => clearInterval(intervalId);
	});
</script>

<MainTemplate mainCenter>
	{#snippet header()}
		<MobileRankerStats player={$userStore.username ?? ''} {rankers} />
	{/snippet}

	{#snippet sidebar()}
		{#if $userStore.username}
			<Brand />
			<SideLeaderboard {rankers} {leaderboardId} />
		{:else}
			<UserSidebar />
		{/if}
	{/snippet}

	{#snippet main()}
		<div class="flex h-full items-center justify-center">
			<div class="my-auto w-full max-w-2xl lg:pb-28">
				<Game
					bind:leaderboardId
					bind:score={currentPlayerScore}
					player={$userStore.username ?? ''}
					playerChainId={chainId as string}
					canStartNewGame={!!$userStore.username}
					showBestScore
					{canMakeMove}
				/>
			</div>
		</div>
		{#if $isHashesListVisible}
			<BlockHashes />
		{/if}
	{/snippet}
</MainTemplate>
