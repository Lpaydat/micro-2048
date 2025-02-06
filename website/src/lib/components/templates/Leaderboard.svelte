<script lang="ts">
	import { onMount } from 'svelte';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import Trash from 'lucide-svelte/icons/trash-2';
	import Star from 'lucide-svelte/icons/star';
	import Plus from 'lucide-svelte/icons/plus';
	import ContinueIcon from 'lucide-svelte/icons/step-forward';
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
	import { addShards, getRandomShard, getShards } from '$lib/stores/shards';
	import { getBoard } from '$lib/graphql/queries/getBoard';

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
				shardIds
			}
			balance
		}
	`;

	const mainClient = getContextClient();
	const leaderboardClient = getClient(leaderboardId, true);
	const playerClient = $derived(getClient($userStore.chainId, true));

	const leaderboard = $derived(
		queryStore({
			client: leaderboardClient,
			query: LEADERBOARD
		})
	);
	const currentBoardId = $derived(getBoardId(leaderboardId));
	const board = $derived(getBoard(playerClient));

	const isEnded = $derived(
		Number($leaderboard?.data?.leaderboard?.endTime ?? '0') - Date.now() < 0
	);
	const isStarted = $derived(
		Number($leaderboard?.data?.leaderboard?.startTime ?? '0') - Date.now() < 0
	);
	const canDeleteEvent = $derived(
		($leaderboard?.data?.leaderboard?.host === $userStore.username || $userStore.isMod) && !isEnded
	);
	const balance = $derived($leaderboard.data?.balance);
	const canPinEvent = $derived($userStore.isMod && !isEnded);
	const canPlayGame = $derived(isStarted && !isEnded && $userStore.username);
	const isPinned = $derived($leaderboard?.data?.leaderboard?.isPinned);

	let newGameAt = $state(Date.now());
	let isNewGameCreated = $state(false);

	const newEventGame = async () => {
		if (isNewGameCreated) return;
		if (!leaderboardId || !$userStore.username) return;

		const shardId = await getRandomShard(leaderboardId, $userStore.username);
		if (!shardId) return;

		newGameAt = Date.now();
		isNewGameCreated = true;
		await newGameBoard(leaderboardId, shardId, newGameAt.toString());
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

	const createShards = () => {
		if (
			leaderboardId &&
			!$leaderboard.fetching &&
			!$leaderboard.data?.leaderboard?.shardIds?.length
		) {
			Array.from({ length: 16 }).forEach(() => {
				newShard(leaderboardClient);
			});
		}
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
				console.log('new game found');
				newGameAt = 0;
				const url = new URL('/game', window.location.origin);
				url.searchParams.set('boardId', $board?.data?.board?.boardId);
				url.searchParams.set('leaderboardId', leaderboardId);

				setBoardId($board.data?.board?.boardId, leaderboardId);
				goto(url.toString(), { replaceState: false });
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
			<RankerLeaderboard
				rankers={$leaderboard.data?.leaderboard?.rankers}
				{balance}
				{...$leaderboard.data?.leaderboard}
			/>
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
					{#if $userStore.isMod}
						<button type="button" class="btn-icon" onclick={createShards}>
							<Plus size={20} />
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
						<ActionButton label="NEW GAME" onclick={newEventGame} disabled={isNewGameCreated} />
					{/if}
				{/snippet}
			</PageHeader>
			<RankerLeaderboard
				rankers={$leaderboard.data?.leaderboard?.rankers}
				{balance}
				hasSubHeader
				{...$leaderboard.data?.leaderboard}
			/>
		{/if}
	{/snippet}
</MainTemplate>
