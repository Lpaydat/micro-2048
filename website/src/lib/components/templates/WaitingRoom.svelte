<script lang="ts">
	import { gql, subscriptionStore } from '@urql/svelte';
	import { onDestroy, onMount } from 'svelte';
	import { page } from '$app/stores';
	import HelpCircle from 'lucide-svelte/icons/circle-help';

	import HelpButton from '../atoms/HelpButton.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import PageHeader from '../molecules/PageHeader.svelte';
	import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import WaitingPlayers from '../organisms/WaitingPlayers.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import { joinGame, leaveGame, startGame, endGame } from '$lib/graphql/mutations';
	import { goto } from '$app/navigation';
	import { getGameDetails } from '$lib/graphql/queries/getGameDetails';
	import { userStore } from '$lib/stores/userStore';
	import { getModalStore, type ModalSettings, type ModalStore } from '@skeletonlabs/skeleton';
	import { getClient } from '$lib/client';

	const modalStore: ModalStore = getModalStore();
	const gameId = $derived($page.params.gameId);
	const autoJoin = $page.url.searchParams.get('autoJoin') === 'true';
	const client = $derived(getClient(gameId));
	const minimumPlayers = 1;

	const GAME_PING_SUBSCRIPTION = gql`
		subscription Notifications($chainId: ID!) {
			notifications(chainId: $chainId)
		}
	`;

	// Subscription for notifications
	const gameMessages = $derived(
		subscriptionStore({
			client,
			query: GAME_PING_SUBSCRIPTION,
			variables: { chainId: gameId }
		})
	);

	const game = $derived(getGameDetails(client));
	const data = $derived(
		$game.data?.eliminationGame && !$game.fetching
			? {
					...$game.data.eliminationGame,
					playerCount: $game.data?.eliminationGame?.players?.length ?? 1
				}
			: undefined
	);

	const username = $derived($userStore.username);
	const isHost = $derived(data?.host === username);
	const isJoined = $derived(data?.players.includes(username));
	const canJoinGame = $derived(data?.playerCount < data?.maxPlayers && !isJoined);
	const prevPage = $derived(data?.status === 'Ended' || !isJoined ? '/elimination' : undefined);

	let gameName = $state('Loading...');
	let isLoaded = $state(false);
	$effect(() => {
		if (!isLoaded && !$game.fetching) {
			isLoaded = true;
		}
		gameName = data?.gameName ?? gameName;
	});

	// Reactive statements for block height and game query reexecution
	let blockHeight = $state(0);
	let initialFetch = $state(true);
	let showActionButtons = $state(!autoJoin);
	$effect(() => {
		const bh = $gameMessages.data?.notifications?.reason?.NewBlock?.height;
		if ((bh && bh !== blockHeight) || initialFetch) {
			blockHeight = bh;
			initialFetch = false;
			game.reexecute({ requestPolicy: 'network-only' });

			if (autoJoin && !showActionButtons) {
				showActionButtons = true;
			}
		}
	});

	$effect(() => {
		if (data?.status === 'Active' && isJoined) {
			const playerBoardId = `${gameId}-${data.currentRound}-${username}`;
			goto(`/game/${playerBoardId}`);
		}
	});

	onMount(() => {
		if (autoJoin && !isJoined) {
			handleJoinGame();
		}
	});

	onDestroy(() => {
		gameMessages.pause();
	});

	const handleJoinGame = () => {
		if (!username) return;
		joinGame(client);
	};

	const handleLeaveGame = () => {
		if (!username) return;
		leaveGame(client);
	};

	const handleStartGame = () => {
		if (data?.playerCount < minimumPlayers || !username) return;
		startGame(client);
	};

	const handleEndGame = () => {
		if (!username) return;
		endGame(client);
	};

	const howToPlayModal: ModalSettings = {
		type: 'component',
		component: 'how-to-play-elimination'
	};

	const howToPlay = () => {
		modalStore.trigger(howToPlayModal);
	};
</script>

<MainTemplate>
	{#snippet sidebar()}
		<UserSidebar />
	{/snippet}

	{#snippet main()}
		<PageHeader color="green" title={gameName} {prevPage}>
			{#snippet actions()}
				{#if isLoaded && $userStore.username && data?.status === 'Waiting' && showActionButtons}
					<HelpButton ariaLabel="How to Play" onclick={howToPlay}>
						<HelpCircle size={20} />
					</HelpButton>
					{#if isHost}
						<ActionButton
							label="START"
							onclick={handleStartGame}
							disabled={data?.playerCount < minimumPlayers}
						/>
						<ActionButton label="CANCEL" hoverColor="danger" onclick={handleEndGame} />
					{:else if canJoinGame}
						<ActionButton label="JOIN" onclick={handleJoinGame} />
					{:else if isJoined}
						<ActionButton label="LEAVE" onclick={handleLeaveGame} />
					{/if}
				{/if}
			{/snippet}
		</PageHeader>
		{#if !$game.fetching}
			<GameSettingsDetails {...data} numberA={data.playerCount} numberB={data.maxPlayers} />
			<WaitingPlayers players={data?.players ?? []} />
		{/if}
	{/snippet}
</MainTemplate>
