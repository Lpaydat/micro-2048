<script lang="ts">
	import { subscriptionStore, getContextClient } from "@urql/svelte";
	import Game from "../organisms/Game.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import Leaderboard from '../organisms/Leaderboard.svelte';
	import Brand from '../molecules/Brand.svelte';
	import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import { onDestroy, onMount } from "svelte";
	import { page } from "$app/stores";
	import { getGameDetails } from "$lib/graphql/queries/getGameDetails";
	import { userStore } from "$lib/stores/userStore";
	import { getMessageBlockheight } from "$lib/utils/getMessageBlockheight";
	import { gql } from "urql";
	import { triggerGameEvent } from "$lib/graphql/mutations/triggerGame";
	import { goto } from "$app/navigation";
	import { nextRound } from "$lib/graphql/mutations/nextRound";

    let boardId: string = $page.params.boardId;

    let unsubscribe: any;

    $: {
        unsubscribe = page.subscribe(($page) => {
            boardId = $page.params.boardId;
        });
    }

    onDestroy(() => {
        if (unsubscribe) unsubscribe();
    });

    let [gameId, round, _username, _playerChainId] = $page.params.boardId.split('-');
    $: r = parseInt($page.params.boardId.match(/\-(\d+)\-/)?.[1] || '0');
    $: username = $page.params.boardId.split('-')[2] || '';
    $: chainId = $page.params.boardId.match(/\-[^-]+-([^-]+)$/)?.[1] || '';
    let canMakeMove = username === $userStore.username;

    const client = getContextClient();

    // Determine if the game is multiplayer based on the URL pattern
    const isMultiplayer = boardId.includes('-');

    const GAME_PING_SUBSCRIPTION = gql`
        subscription Notifications($chainId: ID!) {
            notifications(chainId: $chainId)
        }
    `;

    // Subscription for notifications
    const gameMessages = subscriptionStore({
        client,
        query: GAME_PING_SUBSCRIPTION,
        variables: { chainId: gameId },
    });

    const triggerGameEventMutation = () => triggerGameEvent(client, gameId, username);
    const nextRoundMutation = () => nextRound(client, gameId, username);

    // Reactive declarations
    $: game = getGameDetails(client, gameId, r);
    $: data = $game.data?.eliminationGame && !$game.fetching 
        ? {
            ...$game.data.eliminationGame,
            playerCount: $game.data.eliminationGame.players.length
        }
        : {};

    // Reactive variables for component data
    $: lastUpdated = parseInt($game.data?.eliminationGame?.lastUpdatedTime || '0');
    $: currentRound = $game.data?.eliminationGame?.currentRound;
    $: totalRounds = $game.data?.eliminationGame?.totalRounds;
    $: gameLeaderboard = $game.data?.eliminationGame?.gameLeaderboard;
    $: roundLeaderboard = $game.data?.eliminationGame?.roundLeaderboard?.[0];
    $: status = $game.data?.eliminationGame?.status;
    $: isRoundEnded = roundLeaderboard?.players.length === 0;
    $: bh = getMessageBlockheight($gameMessages.data);

    let countdown = 0;
    let countdownInterval: NodeJS.Timeout;
    let intervalId: NodeJS.Timeout;
    let blockHeight: number | undefined = undefined;
    let isTriggered: boolean = false;

    function updateCountdown() {
        const triggerInterval = $game.data?.eliminationGame?.triggerIntervalSeconds || 0;
        
        if (lastUpdated) {
            // Clear existing interval if any
            if (countdownInterval) clearInterval(countdownInterval);
            
            countdown = triggerInterval - Math.floor((Date.now() - lastUpdated) / 1000);
            
            countdownInterval = setInterval(() => {
                countdown = Math.max(0, countdown - 1);

                if (countdown <= 0 && !isTriggered) {
                    triggerGameEventMutation();
                    isTriggered = true;
                    canMakeMove = false;
                } else {
                    canMakeMove = username === $userStore.username;
                }
            }, 1000);
        }
    }

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
        }
    });

    $: {
        // Re-run countdown update when lastUpdated changes
        if (lastUpdated) {
            updateCountdown();
        }
    }

    let currentPlayerScore = 0;
    let nextTarget: string | undefined = undefined;

    $: {
        const player = $userStore.username || username;
        const chainId = $userStore.chainId || _playerChainId;
        const target = `/game/${gameId}-${currentRound}-${player}-${chainId}`;
        if (currentRound && target !== nextTarget && status === 'Active') {
            nextTarget = target;
            isTriggered = false;
            goto(nextTarget);
        }
    }
    $: isEnded = roundLeaderboard?.eliminatedPlayers.some((player: any) => player.username === username);
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
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
    </svelte:fragment>

    <svelte:fragment slot="main">
        {#if isMultiplayer}
            <GameSettingsDetails
                {data}
                numberA={currentRound}
                numberB={totalRounds}
                numberLabel="Round"
            />
            <div class="h-32 flex items-center justify-center pt-10">
                {#if !isRoundEnded}
                    <div class="text-[#776E65] text-7xl font-bold game-font text-center">
                        {countdown >= 0 ? countdown : '...'}
                    </div>
                {:else if status === 'Ended'}
                    <button
                        class="btn variant-soft-surface text-[#F67C5F] text-3xl font-bold game-font text-center"
                        on:click={() => goto('/elimination')}
                    >
                        Lobby
                    </button>
                {:else}
                    <button
                        class="btn variant-soft-surface text-[#F67C5F] text-3xl font-bold game-font text-center"
                        on:click={nextRoundMutation}
                    >
                        Next Round
                    </button>
                {/if}
            </div>
        {/if}
        <div class="flex justify-center items-center pt-8">
            <div class="w-full max-w-2xl pb-28">
                <Game
                    {isMultiplayer}
                    {isEnded}
                    player={username}
                    playerChainId={chainId}
                    boardId={boardId}
                    canStartNewGame={!isMultiplayer}
                    showBestScore={!isMultiplayer}
                    canMakeMove={canMakeMove}
                    bind:score={currentPlayerScore}
                />
            </div>
        </div>
    </svelte:fragment>
</MainTemplate>
