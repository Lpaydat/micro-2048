<script lang="ts">
	import { onMount } from 'svelte';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import Trash from 'lucide-svelte/icons/trash-2';
	import Star from 'lucide-svelte/icons/star';
	import Plus from 'lucide-svelte/icons/plus';
	import GitBranchPlus from 'lucide-svelte/icons/git-branch-plus';
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
	import { addShards, getShards } from '$lib/stores/shards';
	import { getBoard } from '$lib/graphql/queries/getBoard';
	import { requestFaucetMutation } from '$lib/graphql/mutations/requestFaucet';

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
					isEnded
				}
				shardIds
			}
			balance
		}
	`;

	const mainClient = getContextClient();
	// Must be $derived so it updates when leaderboardId changes (navigation between tournaments)
	const leaderboardClient = $derived(getClient(leaderboardId, true));
	const playerClient = $derived(getClient($userStore.chainId, true));

	const leaderboard = $derived(
		queryStore({
			client: leaderboardClient,
			query: LEADERBOARD
		})
	);
	const currentBoardId = $derived(getBoardId(leaderboardId));
	const board = $derived(getBoard(playerClient));

	const isEnded = $derived.by(() => {
		const endTime = Number($leaderboard?.data?.leaderboard?.endTime ?? '0');
		if (endTime === 0) return false;
		return endTime - Date.now() < 0;
	});
	const isStarted = $derived.by(() => {
		const startTime = Number($leaderboard?.data?.leaderboard?.startTime ?? '0');
		if (startTime === 0) return true;
		return startTime - Date.now() < 0;
	});
	const canDeleteEvent = $derived(
		($leaderboard?.data?.leaderboard?.host === $userStore.username || $userStore.isMod) && !isEnded
	);
	const balance = $derived($leaderboard.data?.balance);
	const canPinEvent = $derived($userStore.isMod && !isEnded);
	const canPlayGame = $derived(isStarted && !isEnded && $userStore.username);
	const isPinned = $derived($leaderboard?.data?.leaderboard?.isPinned);

	let newGameAt = $state(Date.now());
	let isNewGameCreated = $state(false);
	let isCreatingNewGame = $state(false);

	const newEventGame = async () => {
		if (isNewGameCreated || isCreatingNewGame) return;
		if (!leaderboardId || !$userStore.username) return;

		isCreatingNewGame = true;
		newGameAt = Date.now();
		isNewGameCreated = true;

		try {
			await newGameBoard(leaderboardId, newGameAt.toString());
		} catch (error) {
			console.error('Failed to create new game:', error);
			isCreatingNewGame = false;
			isNewGameCreated = false;
		}
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

	let isCreatingShards = $state(false);
	const createShards = () => {
		if (
			leaderboardId &&
			!isCreatingShards &&
			!$leaderboard.fetching &&
			!$leaderboard.data?.leaderboard?.shardIds?.length
		) {
			isCreatingShards = true;
			Array.from({ length: 16 }).forEach(() => {
				newShard(leaderboardClient);
			});
		}
	};

	const faucet = () => {
		requestFaucetMutation(leaderboardClient);
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
				// Check if board data is valid and ready
				const boardData = $board?.data?.board;
				const hasValidBoard =
					boardData?.board && Array.isArray(boardData?.board) && boardData?.board.length > 0;

				// Reset states
				isCreatingNewGame = false;
				newGameAt = 0;

				const url = new URL('/game', window.location.origin);
				url.searchParams.set('boardId', $board?.data?.board?.boardId);
				url.searchParams.set('leaderboardId', leaderboardId);

				setBoardId($board.data?.board?.boardId, leaderboardId);

				// Conditional reload: force reload if board data isn't valid
				if (hasValidBoard) {
					// Normal redirect - board data is ready
					goto(url.toString(), { replaceState: false });
				} else {
					// Force reload to ensure fresh data fetch
					goto(url.toString(), { replaceState: false, invalidateAll: true });
				}
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
						<button type="button" class="btn-icon" onclick={faucet}>
							<Plus size={20} />
						</button>
						<button type="button" class="btn-icon" onclick={createShards}>
							<GitBranchPlus size={20} />
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
						<ActionButton
							label={isCreatingNewGame ? 'CREATING...' : 'NEW GAME'}
							onclick={newEventGame}
							disabled={isCreatingNewGame || isNewGameCreated}
						/>
					{/if}
				{/snippet}
			</PageHeader>
			<RankerLeaderboard
				rankers={$leaderboard.data?.leaderboard?.rankers}
				{balance}
				hasSubHeader
				{...$leaderboard.data?.leaderboard}
				{leaderboardId}
			/>
		{/if}
	{/snippet}
</MainTemplate>
