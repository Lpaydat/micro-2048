<script lang="ts">
	import { subscriptionStore, getContextClient } from "@urql/svelte";
	import Game from "../organisms/Game.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import Leaderboard from '../organisms/Leaderboard.svelte';
	import Brand from '../molecules/Brand.svelte';
	import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import { onMount } from "svelte";
	import { page } from "$app/stores";
	import { getGameDetails } from "$lib/graphql/queries/getGameDetails";
	import { userStore } from "$lib/stores/userStore";
	import { getPlayerInfo } from "$lib/graphql/queries/getPlayerInfo";
	import { getMessageBlockheight } from "$lib/utils/getMessageBlockheight";
	import { gql } from "urql";

    const boardId = $page.params.boardId;
    const [gameId, round, username] = boardId.split('-');

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

    // Reactive declarations
    $: player = getPlayerInfo(client, username);
    $: game = getGameDetails(client, gameId, parseInt(round));
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

    let currentPlayerScore = 0;
    let blockHeight = 0;
    $: bh = getMessageBlockheight($gameMessages.data);

    let intervalId: NodeJS.Timeout;

    onMount(() => {
        intervalId = setInterval(() => {
            if (bh && bh !== blockHeight) {
                blockHeight = bh ?? 0;
                game.reexecute({ requestPolicy: 'network-only' });
            }
        }, 250);

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
                    playerChainId={$player.data?.player?.chainId}
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
