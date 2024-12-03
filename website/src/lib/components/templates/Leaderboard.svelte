<script lang="ts">
	import { onMount } from 'svelte';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import Trash from 'lucide-svelte/icons/trash-2';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import RankerLeaderboard from '../organisms/RankerLeaderboard.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import PageHeader from '../molecules/PageHeader.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import { userStore } from '$lib/stores/userStore';
	import { hashSeed } from '$lib/utils/random';
	import { goto } from '$app/navigation';
	import { newGame } from '$lib/graphql/mutations/newBoard';
	import { setGameCreationStatus } from '$lib/stores/gameStore';
	import { getBoardId, setBoardId } from '$lib/stores/boardId';
	import { deleteEvent } from '$lib/graphql/mutations/createEvent';

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
				description
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

	const currentBoardId = $derived(getBoardId(leaderboardId));

	// Sort the rankers by score in descending order
	const sortedRankers = $derived(
		$leaderboard?.data?.leaderboard?.rankers.slice().sort((a: any, b: any) => b.score - a.score)
	);
	const canDeleteEvent = $derived(
		$leaderboard?.data?.leaderboard?.host === $userStore.username &&
			$leaderboard?.data?.leaderboard?.totalBoards === 0
	);

	const newEventGame = async () => {
		if (!leaderboardId) return;
		if (!$userStore.username) return;

		const seed = Math.floor(Math.random() * 10_000_000).toString();
		const timestamp = Date.now().toString();
		const boardId = (await hashSeed(seed, $userStore.username, timestamp)).toString();
		setBoardId(boardId, leaderboardId);
		setGameCreationStatus(true);
		newGame(client, seed, timestamp, leaderboardId);

		const url = new URL('/game', window.location.origin);
		url.searchParams.set('boardId', boardId);
		goto(url.toString(), { replaceState: false });
	};

	const deleteEventGame = () => {
		deleteEvent(client, leaderboardId);
		setTimeout(() => {
			goto('/events');
		}, 100);
	};

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
				{#snippet subActions()}
					{#if canDeleteEvent}
						<button type="button" class="btn" onclick={deleteEventGame}>
							<Trash size={20} />
						</button>
					{/if}
				{/snippet}
				{#snippet actions()}
					{#if currentBoardId}
						<a href={`/game/?boardId=${currentBoardId}`}>
							<ActionButton icon="plus" label="RESUME GAME" />
						</a>
					{/if}
					<ActionButton icon="plus" label="NEW GAME" onclick={newEventGame} />
				{/snippet}
			</PageHeader>
			<RankerLeaderboard rankers={sortedRankers} {...$leaderboard.data?.leaderboard} />
		{/if}
	{/snippet}
</MainTemplate>
