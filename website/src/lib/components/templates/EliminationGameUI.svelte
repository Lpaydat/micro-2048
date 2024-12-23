<script lang="ts">
	import { subscriptionStore } from '@urql/svelte';
	import BlockHashes from '../molecules/BlockHashes.svelte';
	import Game from '../organisms/Game.svelte';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import EliminationLeaderboard from '../organisms/EliminationLeaderboard.svelte';
	import Brand from '../molecules/Brand.svelte';
	import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import { onDestroy, onMount } from 'svelte';
	import { page } from '$app/stores';
	import { getGameDetails } from '$lib/graphql/queries/getGameDetails';
	import { userStore } from '$lib/stores/userStore';
	import { getMessageBlockheight } from '$lib/utils/getMessageBlockheight';
	import { gql } from 'urql';
	import { goto } from '$app/navigation';
	import { nextRound, triggerGame } from '$lib/graphql/mutations';
	import { isHashesListVisible } from '$lib/stores/hashesStore';
	import RoundButton from '../molecules/RoundButton.svelte';
	import MobileUserStats from '../organisms/MobileEliminationStats.svelte';
	import { getClient } from '$lib/client';

	let boardId = $state<string>($page.params.boardId);

	let unsubscribe: any;

	$effect(() => {
		unsubscribe = page.subscribe(($page) => {
			boardId = $page.params.boardId;
		});
	});

	onDestroy(() => {
		if (unsubscribe) unsubscribe();
	});

	const [gameId, playerChainId, _username, round] = $derived($page.params.boardId.split('-'));
	const r = $derived(parseInt(round || '0'));
	const username = $derived($page.params.boardId.split('-')[2] || '');
	const isBoardOwner = $derived(username === $userStore.username);

	const client = $derived(getClient(gameId));

	// Determine if the game is multiplayer based on the URL pattern
	const isMultiplayer = $derived(boardId.includes('-'));

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

	const triggerGameEventMutation = () => triggerGame(client);
	const nextRoundMutation = () => nextRound(client);

	// Reactive declarations
	const game = $derived(getGameDetails(client, r));
	const data = $derived(
		$game.data?.eliminationGame && !$game.fetching
			? {
					...$game.data.eliminationGame,
					playerCount: $game.data.eliminationGame.players?.length || 0
				}
			: {}
	);

	// Reactive variables for component data
	const lastUpdated = $derived(parseInt($game.data?.eliminationGame?.lastUpdatedTime || '0'));
	const currentRound = $derived($game.data?.eliminationGame?.currentRound);
	const totalRounds = $derived($game.data?.eliminationGame?.totalRounds);
	const gameLeaderboard = $derived($game.data?.eliminationGame?.gameLeaderboard);
	const roundLeaderboard = $derived($game.data?.eliminationGame?.roundLeaderboard?.[0]);
	const status = $derived($game.data?.eliminationGame?.status);
	const isRoundEnded = $derived(roundLeaderboard?.players.length === 0);
	const bh = $derived(getMessageBlockheight($gameMessages.data));

	let countdown = $state(0);
	let countdownInterval = $state<NodeJS.Timeout | undefined>(undefined);
	let intervalId = $state<NodeJS.Timeout | undefined>(undefined);
	let blockHeight = $state<number | undefined>(undefined);
	let isTriggered = $state(false);
	let currentPlayerScore = $state(0);
	let nextTarget = $state<string | undefined>(undefined);

	let lastUpdatedRecord = 0;

	const handleEliminationTrigger = () => {
		if (countdown <= 0 && !isTriggered) {
			triggerGameEventMutation();
			isTriggered = true;
		}
	};

	const handleMoveCallback = () => {
		handleEliminationTrigger();
	};

	const updateCountdown = () => {
		const triggerInterval = $game.data?.eliminationGame?.triggerIntervalSeconds || 0;

		if (lastUpdated) {
			// Clear existing interval if any
			if (countdownInterval) clearInterval(countdownInterval);

			countdown = triggerInterval - Math.floor((Date.now() - lastUpdated) / 1000);

			countdownInterval = setInterval(() => {
				countdown = Math.max(0, countdown - 1);
				handleEliminationTrigger();
			}, 1000);
		}
	};

	onMount(() => {
		updateCountdown();

		intervalId = setInterval(() => {
			if (bh !== blockHeight) {
				blockHeight = bh;
				game.reexecute({ requestPolicy: 'network-only' });
			}
		}, 1000);

		return () => {
			clearInterval(intervalId);
			if (countdownInterval) clearInterval(countdownInterval);
			gameMessages.pause();
		};
	});

	$effect(() => {
		if (lastUpdated !== lastUpdatedRecord) {
			lastUpdatedRecord = lastUpdated;
			updateCountdown();
		}
	});

	$effect(() => {
		const player = $userStore.username || username;
		const target = `/game/${gameId}-${$userStore.chainId}-${player}-${currentRound}`;
		if (currentRound && target !== nextTarget && status === 'Active') {
			nextTarget = target;
			isTriggered = false;
			goto(nextTarget);
		}
	});

	const isEliminated = $derived(
		roundLeaderboard?.eliminatedPlayers.some((player: any) => player.username === username)
	);

	let windowWidth = $state(0);
	let shouldRenderGameDetails = $state(false);
	$effect(() => {
		if (data?.gameName) shouldRenderGameDetails = true;
	});
</script>

<MainTemplate bind:windowWidth mainCenter>
	{#snippet header()}
		<MobileUserStats
			player={username}
			{currentRound}
			{roundLeaderboard}
			{gameLeaderboard}
			{currentPlayerScore}
		/>
	{/snippet}

	{#snippet sidebar()}
		{#if isMultiplayer && $userStore.username}
			<Brand />
			<EliminationLeaderboard
				player={username}
				{currentRound}
				{gameLeaderboard}
				{roundLeaderboard}
				{currentPlayerScore}
			/>
		{:else}
			<UserSidebar />
		{/if}
	{/snippet}

	{#snippet main()}
		<div class="flex flex-1 flex-col items-stretch">
			{#if isMultiplayer}
				{#if windowWidth > 768 && shouldRenderGameDetails}
					<GameSettingsDetails
						{...data}
						numberA={currentRound}
						numberB={totalRounds}
						numberLabel="Round"
					/>
				{/if}
				<RoundButton {isRoundEnded} {countdown} {status} onclick={nextRoundMutation} />
			{/if}
			<div class="flex items-center justify-center pt-2 lg:pt-6 xl:pt-8">
				<div class="my-auto w-full max-w-2xl">
					<Game
						{isMultiplayer}
						isEnded={isEliminated}
						player={username}
						{boardId}
						chainId={playerChainId}
						canStartNewGame={!isMultiplayer}
						showBestScore={!isMultiplayer}
						canMakeMove={isBoardOwner && !isEliminated}
						bind:score={currentPlayerScore}
						on:move={handleMoveCallback}
					/>
				</div>
			</div>
		</div>
		{#if $isHashesListVisible}
			<BlockHashes />
		{/if}
	{/snippet}

	{#snippet footer()}
		{#if isMultiplayer && windowWidth <= 768 && shouldRenderGameDetails}
			<GameSettingsDetails
				{...data}
				numberA={currentRound}
				numberB={totalRounds}
				numberLabel="Round"
			/>
		{/if}
	{/snippet}
</MainTemplate>
