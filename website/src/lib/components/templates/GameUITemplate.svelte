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

    // export let activeRound: number;
    let boardId: string = $page.params.boardId;

    let unsubscribe: any;

    onDestroy(() => {
        if (unsubscribe) unsubscribe()
    })

    $: {
        const unsubscribe = page.subscribe(($page) => {
            boardId = $page.params.boardId;
        })

        onDestroy(() => {
            if (unsubscribe) unsubscribe()
        })
    }

    let [gameId, round, username, playerChainId] = boardId.split('-');
    $: r = parseInt($page.params.boardId.match(/\-(\d+)\-/)?.[1] || '0');

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
    $: currentRound = $game.data?.eliminationGame?.currentRound;
    $: totalRounds = $game.data?.eliminationGame?.totalRounds;
    $: gameLeaderboard = $game.data?.eliminationGame?.gameLeaderboard;
    $: roundLeaderboard = $game.data?.eliminationGame?.roundLeaderboard?.[0];
    $: status = $game.data?.eliminationGame?.status;

    let currentPlayerScore = 0;

    let nextTarget: string | undefined = undefined;

    $: {
        const target = `/game/${gameId}-${currentRound}-${username}-${playerChainId}`;
        if (currentRound && target !== nextTarget && status === 'Active') {
            nextTarget = target;
            goto(nextTarget);
        }
    }

    let intervalId: NodeJS.Timeout;
    let blockHeight: number | undefined = undefined;
    $: bh = getMessageBlockheight($gameMessages.data);

    onMount(() => {
        intervalId = setInterval(() => {
            if ((bh !== blockHeight)) {
                blockHeight = bh;
                game.reexecute({ requestPolicy: 'network-only' });
            }
        }, 1000);

        return () => {
            clearInterval(intervalId);
            gameMessages.pause();
        }
    });
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        {#if isMultiplayer && $userStore.username}
            <Brand />
            <button class='text-white' on:click={triggerGameEventMutation}>Trigger Game Event</button>
            <button class='text-white' on:click={nextRoundMutation}>Next Round</button>
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
        {/if}
        <div class="flex justify-center items-center h-full">
            <div class="w-full max-w-2xl pb-28">
                <Game
                    player={username}
                    playerChainId={playerChainId}
                    boardId={boardId}
                    canStartNewGame={!isMultiplayer}
                    showBestScore={!isMultiplayer}
                    canMakeMove={username === $userStore.username}
                    bind:score={currentPlayerScore}
                />
            </div>
        </div>
    </svelte:fragment>
</MainTemplate>
