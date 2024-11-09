<script lang="ts">
	import { gql, subscriptionStore, getContextClient } from "@urql/svelte";
	import Game from "../organisms/Game.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import Leaderboard from '../organisms/Leaderboard.svelte';
	import Brand from '../molecules/Brand.svelte';
	import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import { onDestroy } from "svelte";
	import { page } from "$app/stores";
	import { getGameDetails } from "$lib/graphql/queries/getGameDetails";
	import { userStore } from "$lib/stores/userStore";
	import { getPlayerInfo } from "$lib/graphql/queries/getPlayerInfo";

    const boardId = $page.params.boardId;
    const [gameId, round, username] = boardId.split('-');

    const GAME_PING_SUBSCRIPTION = gql`
        subscription Notifications($chainId: ID!) {
            notifications(chainId: $chainId)
        }
    `;

    const client = getContextClient();

    // Subscription for notifications
    const gameMessages = subscriptionStore({
        client,
        query: GAME_PING_SUBSCRIPTION,
        variables: { chainId: gameId },
    });

    $: player = getPlayerInfo(client, username);
    $: game = getGameDetails(client, gameId, parseInt(round));
    $: data = $game.data?.eliminationGame && !$game.fetching 
        ? {
            ...$game.data.eliminationGame,
            playerCount: $game.data.eliminationGame.players.length
        }
        : {};

    const isMultiplayer = boardId.includes('-');
    let currentPlayerScore = 0;
    let lastHash = '';


    // Check for new game messages every second
    setInterval(() => {
        if ($gameMessages.data?.notifications?.reason?.NewBlock?.hash && 
            $gameMessages.data.notifications.reason.NewBlock.hash !== lastHash) {
            lastHash = $gameMessages.data.notifications.reason.NewBlock.hash;
            game.reexecute({ requestPolicy: 'network-only' });
        }
    }, 1000);

    onDestroy(() => {
        gameMessages.pause();
    });
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        {#if isMultiplayer}
            <Brand />
            <Leaderboard
                player={username}
                currentRound={data?.currentRound}
                gameLeaderboard={data?.gameLeaderboard}
                roundLeaderboard={data?.roundLeaderboard?.[0]}
                {currentPlayerScore}
            />
        {:else}
            <UserSidebar username={$userStore.username} />
        {/if}
    </svelte:fragment>

    <svelte:fragment slot="main">
        {#if isMultiplayer}
            <GameSettingsDetails
                {data}
                numberA={data?.currentRound}
                numberB={data?.totalRounds}
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
