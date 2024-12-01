<script lang="ts">
	import { getContextClient, gql, subscriptionStore } from '@urql/svelte';
	import { onDestroy } from 'svelte';
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

	const modalStore: ModalStore = getModalStore();
	const client = getContextClient();
	const gameId = $derived($page.params.gameId);
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

	const game = $derived(getGameDetails(client, gameId));
	const data = $derived(
		$game.data?.eliminationGame && !$game.fetching
			? {
					...$game.data.eliminationGame,
					playerCount: $game.data.eliminationGame.players.length
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
	$effect(() => {
		const bh = $gameMessages.data?.notifications?.reason?.NewIncomingBundle?.height;
		if ((bh && bh !== blockHeight) || initialFetch) {
			blockHeight = bh;
			initialFetch = false;
			game.reexecute({ requestPolicy: 'network-only' });
		}
	});

	$effect(() => {
		if (data?.status === 'Active' && isJoined) {
			const playerBoardId = `${gameId}-${data.currentRound}-${username}-${$userStore.chainId}`;
			goto(`/game/${playerBoardId}`);
		}
	});

	onDestroy(() => {
		gameMessages.pause();
	});

	const handleJoinGame = () => {
		if (!username) return;
		joinGame(client, gameId);
	};

	const handleLeaveGame = () => {
		if (!username) return;
		leaveGame(client, gameId);
	};

	const handleStartGame = () => {
		if (data?.playerCount < minimumPlayers || !username) return;
		startGame(client, gameId);
	};

	const handleEndGame = () => {
		if (!username) return;
		endGame(client, gameId);
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
				{#if isLoaded && $userStore.username && data?.status === 'Waiting'}
					<HelpButton ariaLabel="How to Play" onclick={howToPlay}>
						<HelpCircle size={20} />
					</HelpButton>
					{#if isHost}
						<ActionButton
							label="START"
							onclick={handleStartGame}
							disabled={data.playerCount < minimumPlayers}
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
