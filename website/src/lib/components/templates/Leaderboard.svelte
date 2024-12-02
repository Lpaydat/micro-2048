<script lang="ts">
	import { onMount } from 'svelte';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import RankerLeaderboard from '../organisms/RankerLeaderboard.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import PageHeader from '../molecules/PageHeader.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import { userStore } from '$lib/stores/userStore';
	import { hashSeed } from '$lib/utils/random';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { newGame } from '$lib/graphql/mutations/newBoard';
	import { setGameCreationStatus } from '$lib/stores/gameStore';

	interface Props {
		leaderboardId?: string;
		prevPage?: string;
	}

	let { leaderboardId = '', prevPage }: Props = $props();

	const LEADERBOARD = gql`
		query Leaderboard($leaderboardId: String!) {
			leaderboard(leaderboardId: $leaderboardId) {
				leaderboardId
				name
				# description
				host
				startTime
				endTime
				totalBoards
				totalPlayers
				rankers {
					username
					score
					boardId
				}
			}
		}
	`;

	const client = getContextClient();

	const leaderboard = $derived(
		queryStore({
			client,
			query: LEADERBOARD,
			variables: { leaderboardId }
		})
	);

	// Sort the rankers by score in descending order
	const sortedRankers = $derived(
		$leaderboard.data?.leaderboard.rankers.slice().sort((a: any, b: any) => b.score - a.score)
	);

	const newEventGame = async () => {
		if (!leaderboardId) return;
		if (!$userStore.username) return;

		const seed = Math.floor(Math.random() * 10_000_000).toString();
		const timestamp = Date.now().toString();
		const boardId = (await hashSeed(seed, $userStore.username, timestamp)).toString();
		localStorage.setItem('boardId', boardId);
		setGameCreationStatus(true);
		newGame(client, seed, timestamp, leaderboardId);

		const url = new URL('/game', window.location.origin);
		url.searchParams.set('boardId', boardId);
		url.searchParams.set('leaderboardId', leaderboardId);
		goto(url.toString());
	}

	onMount(() => {
		leaderboard.reexecute({ requestPolicy: 'network-only' });

		const interval = setInterval(() => {
			leaderboard.reexecute({ requestPolicy: 'network-only' });
		}, 5000);

		return () => clearInterval(interval);
	});
</script>

<MainTemplate>
	{#snippet sidebar()}
		<UserSidebar />
	{/snippet}

	{#snippet main()}
		{#if !leaderboardId}
			<RankerLeaderboard rankers={sortedRankers} />
		{:else}
			<PageHeader title={$leaderboard?.data?.leaderboard?.name} {prevPage}>
				{#snippet actions()}
					<ActionButton icon="plus" label="NEW GAME" onclick={newEventGame} />
				{/snippet}
			</PageHeader>
			<RankerLeaderboard rankers={sortedRankers} />
		{/if}
	{/snippet}
</MainTemplate>
