<script lang="ts">
	import { subscriptionStore, getContextClient } from '@urql/svelte';
	import BlockHashes from '../molecules/BlockHashes.svelte';
	import Game from '../organisms/Game.svelte';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import Leaderboard from '../organisms/Leaderboard.svelte';
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
	import MobileUserStats from '../organisms/MobileUserStats.svelte';

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

	const [gameId, _, _username, playerChainId] = $derived($page.params.boardId.split('-'));
	const r = $derived(parseInt($page.params.boardId.match(/\-(\d+)\-/)?.[1] || '0'));
	const username = $derived($page.params.boardId.split('-')[2] || '');
	const chainId = $derived($page.params.boardId.match(/\-[^-]+-([^-]+)$/)?.[1] || '');
	let canMakeMove = $state(false);
	let initCanMakeMove = $state(false);

	$effect(() => {
		if (!initCanMakeMove) {
			initCanMakeMove = true;
			canMakeMove = username === $userStore.username;
		}
	});

	const client = getContextClient();

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

	const triggerGameEventMutation = () => triggerGame(client, gameId);
	const nextRoundMutation = () => nextRound(client, gameId);

	// Reactive declarations
	const game = $derived(getGameDetails(client, gameId, r));
	const data = $derived(
		$game.data?.eliminationGame && !$game.fetching
			? {
					...$game.data.eliminationGame,
					playerCount: $game.data.eliminationGame.players.length
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
			canMakeMove = false;
		} else {
			canMakeMove = username === $userStore.username;
		}
	}

	const handleMoveCallback = () => {
		handleEliminationTrigger();
	}

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
		const chainId = $userStore.chainId || playerChainId;
		const target = `/game/${gameId}-${currentRound}-${player}-${chainId}`;
		if (currentRound && target !== nextTarget && status === 'Active') {
			nextTarget = target;
			// TODO: this line make eliminated players don't trigger the game
			isTriggered = false;
			goto(nextTarget);
		}
	});

	const isEnded = $derived(
		roundLeaderboard?.eliminatedPlayers.some((player: any) => player.username === username)
	);

	let windowWidth = $state(0);
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
			<Leaderboard
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
				{#if windowWidth > 768 && data?.gameName}
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
				<div class="w-full max-w-2xl xl:pb-28">
					<Game
						{isMultiplayer}
						{isEnded}
						player={username}
						playerChainId={chainId}
						{boardId}
						canStartNewGame={!isMultiplayer}
						showBestScore={!isMultiplayer}
						canMakeMove={canMakeMove && !isEnded}
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
		{#if isMultiplayer && windowWidth <= 768 && data?.gameName}
			<GameSettingsDetails
				{...data}
				numberA={currentRound}
				numberB={totalRounds}
				numberLabel="Round"
			/>
		{/if}
	{/snippet}
</MainTemplate>
