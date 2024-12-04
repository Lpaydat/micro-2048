<script lang="ts">
	import { onMount } from 'svelte';
	import { Client, getContextClient, gql, queryStore } from '@urql/svelte';
	import Trash from 'lucide-svelte/icons/trash-2';
	import Star from 'lucide-svelte/icons/star';
	import ContinueIcon from 'lucide-svelte/icons/step-forward';
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
	import { deleteEvent, togglePinEvent } from '$lib/graphql/mutations/leaderboardAction.ts';
	import { getModalStore, type ModalSettings } from '@skeletonlabs/skeleton';

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
				isPinned
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
	const isEnded = $derived(
		Number($leaderboard?.data?.leaderboard?.endTime ?? '0') - Date.now() < 0
	);
	const isStarted = $derived(
		Number($leaderboard?.data?.leaderboard?.startTime ?? '0') - Date.now() < 0
	);
	const canDeleteEvent = $derived(
		($leaderboard?.data?.leaderboard?.host === $userStore.username || $userStore.isAdmin) &&
			!isEnded
	);
	const canPinEvent = $derived($userStore.isAdmin && !isEnded);
	const canPlayGame = $derived(isStarted && !isEnded && $userStore.username);
	const isPinned = $derived($leaderboard?.data?.leaderboard?.isPinned);

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
		url.searchParams.set('leaderboardId', leaderboardId);
		setTimeout(() => {
			goto(url.toString(), { replaceState: false });
		}, 1000);
	};

	const deleteEventGame = () => {
		deleteEvent(client, leaderboardId);
		setTimeout(() => {
			goto('/events');
		}, 250);
	};

	const togglePin = () => {
		togglePinEvent(client, leaderboardId);
		setTimeout(() => {
			leaderboard.reexecute({ requestPolicy: 'network-only' });
		}, 500);
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
		leaderboard.reexecute({ requestPolicy: 'network-only' });

		const interval = setInterval(() => {
			leaderboard.reexecute({ requestPolicy: 'network-only' });
		}, 5000);

		return () => clearInterval(interval);
	});

	onMount(() => {
		updateSize();
		window.addEventListener('resize', updateSize);
		return () => window.removeEventListener('resize', updateSize);
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
						<ActionButton label="NEW GAME" onclick={newEventGame} />
					{/if}
				{/snippet}
			</PageHeader>
			<RankerLeaderboard rankers={sortedRankers} {...$leaderboard.data?.leaderboard} />
		{/if}
	{/snippet}
</MainTemplate>
