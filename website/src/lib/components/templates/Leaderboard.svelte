<script lang="ts">
	import { onMount } from 'svelte';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import Trash from 'lucide-svelte/icons/trash-2';
	import Star from 'lucide-svelte/icons/star';
	import ContinueIcon from 'lucide-svelte/icons/step-forward';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import RankerLeaderboard from '../organisms/RankerLeaderboard.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import PageHeader from '../molecules/PageHeader.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import { userStore } from '$lib/stores/userStore';
	import { goto } from '$app/navigation';
	import { getBoardId, setBoardId } from '$lib/stores/boardId';
	import { deleteEvent, togglePinEvent } from '$lib/graphql/mutations/leaderboardAction.ts';
	import { getModalStore, type ModalSettings } from '@skeletonlabs/skeleton';
	import { getClient } from '$lib/client';
	import { newGameBoard } from '$lib/game/newGameBoard';

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
				rankers {
					username
					score
					boardId
				}
			}
		}
	`;

	const mainClient = getContextClient();
	const leaderboardClient = getClient(leaderboardId, true);

	const leaderboard = $derived(
		queryStore({
			client: leaderboardClient,
			query: LEADERBOARD
		})
	);

	const currentBoardId = $derived(getBoardId(leaderboardId));

	const isEnded = $derived(
		Number($leaderboard?.data?.leaderboard?.endTime ?? '0') - Date.now() < 0
	);
	const isStarted = $derived(
		Number($leaderboard?.data?.leaderboard?.startTime ?? '0') - Date.now() < 0
	);
	const canDeleteEvent = $derived(
		($leaderboard?.data?.leaderboard?.host === $userStore.username || $userStore.isMod) && !isEnded
	);
	const canPinEvent = $derived($userStore.isMod && !isEnded);
	const canPlayGame = $derived(isStarted && !isEnded && $userStore.username);
	const isPinned = $derived($leaderboard?.data?.leaderboard?.isPinned);

	const newEventGame = async () => {
		if (!leaderboardId || !$userStore.username) return;
		const boardId = await newGameBoard(leaderboardId);

		const url = new URL('/game', window.location.origin);
		url.searchParams.set('boardId', boardId);
		url.searchParams.set('leaderboardId', leaderboardId);
		setTimeout(() => {
			goto(url.toString(), { replaceState: false });
		}, 1000);
	};

	const deleteEventGame = () => {
		deleteEvent(mainClient, leaderboardId);
		setTimeout(() => {
			goto('/events');
		}, 250);
	};

	const togglePin = () => {
		togglePinEvent(mainClient, leaderboardId);
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
		}, updateInterval);

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
			<RankerLeaderboard rankers={$leaderboard?.data?.leaderboard?.rankers} />
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
			<RankerLeaderboard
				rankers={$leaderboard?.data?.leaderboard?.rankers}
				hasSubHeader
				{...$leaderboard.data?.leaderboard}
			/>
		{/if}
	{/snippet}
</MainTemplate>
