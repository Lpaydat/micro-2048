<script lang="ts">
	import { getContextClient, gql, subscriptionStore } from '@urql/svelte';
	import { onDestroy } from 'svelte';
	import Clock from 'lucide-svelte/icons/clock-4';
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

	const gameId = $page.params.gameId;
	const minimumPlayers = 1;

	const modalStore: ModalStore = getModalStore();
	const client = getContextClient();
	$: username = $userStore.username;

	const GAME_PING_SUBSCRIPTION = gql`
		subscription Notifications($chainId: ID!) {
			notifications(chainId: $chainId)
		}
	`;

	// Subscription for notifications
	const gameMessages = subscriptionStore({
		client,
		query: GAME_PING_SUBSCRIPTION,
		variables: { chainId: gameId }
	});

	$: game = getGameDetails(client, gameId);
	$: data =
		$game.data?.eliminationGame && !$game.fetching
			? {
					...$game.data.eliminationGame,
					playerCount: $game.data.eliminationGame.players.length
				}
			: undefined;

	$: if (data?.status === 'Active' && isJoined) {
		const playerBoardId = `${gameId}-${data.currentRound}-${username}-${$userStore.chainId}`;
		goto(`/game/${playerBoardId}`);
	}

	$: isHost = data?.host === username;
	$: isJoined = data?.players.includes(username);
	$: canJoinGame = data?.playerCount < data?.maxPlayers && !isJoined;

	let gameName = 'Loading...';
	let isLoaded = false;
	$: if (!isLoaded && !$game.fetching) {
		isLoaded = true;
	}
	$: gameName = data?.gameName ?? gameName;

	// Reactive statements for block height and game query reexecution
	let blockHeight = 0;
	$: bh = $gameMessages.data?.notifications?.reason?.NewIncomingBundle?.height;
	$: if (bh && bh !== blockHeight) {
		blockHeight = bh;
		game.reexecute({ requestPolicy: 'network-only' });
	}

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

	$: prevPage = data?.status === 'Ended' ? '/elimination' : undefined;
</script>

<MainTemplate>
	<svelte:fragment slot="sidebar">
		<UserSidebar />
	</svelte:fragment>

	<svelte:fragment slot="main">
		<PageHeader color="green" title={gameName} {prevPage}>
			<svelte:fragment slot="actions">
				{#if isLoaded && $userStore.username && data?.status === 'Waiting'}
					<HelpButton ariaLabel="How to Play" on:click={howToPlay}>
						<HelpCircle size={20} />
					</HelpButton>
					{#if isHost}
						<ActionButton
							label="START"
							on:click={handleStartGame}
							disabled={data.playerCount < minimumPlayers}
						/>
						<ActionButton label="CANCEL" hoverColor="danger" on:click={handleEndGame} />
					{:else if canJoinGame}
						<ActionButton label="JOIN" on:click={handleJoinGame} />
					{:else if isJoined}
						<ActionButton label="LEAVE" on:click={handleLeaveGame} />
					{/if}
				{/if}
			</svelte:fragment>
		</PageHeader>
		{#if !$game.fetching}
			<GameSettingsDetails {data} numberA={data.playerCount} numberB={data.maxPlayers} />
			<WaitingPlayers players={data?.players ?? []} />
		{/if}
	</svelte:fragment>
</MainTemplate>
