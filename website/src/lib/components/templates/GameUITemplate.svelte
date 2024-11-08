<script lang="ts">
	import { gql, subscriptionStore, getContextClient } from "@urql/svelte";
	import Game from "../organisms/Game.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import Leaderboard from '../organisms/Leaderboard.svelte';
	import Brand from '../molecules/Brand.svelte';
	import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import { getContext } from "svelte";
	import { page } from "$app/stores";
	import { getGameDetails } from "$lib/graphql/queries/getGameDetails";

    let { username, chainId: playerChainId }: { username: string, chainId: string } = getContext('player');
    const boardId = $page.params.boardId;
    const [gameId, round, player] = boardId.split('-');

    const GAME_PING_SUBSCRIPTION = gql`
        subscription Notifications($chainId: ID!) {
            notifications(chainId: $chainId)
        }
    `;

    const PLAYER_PING_SUBSCRIPTION = gql`
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

    const playerMessages = subscriptionStore({
        client,
        query: PLAYER_PING_SUBSCRIPTION,
        variables: { chainId: playerChainId },
    });

    $: console.log('gameId', gameId);
    $: game = getGameDetails(client, gameId, parseInt(round));
    $: data = $game.data?.eliminationGame && !$game.fetching 
        ? {
            ...$game.data.eliminationGame,
            playerCount: $game.data.eliminationGame.players.length
        }
        : {};

    $: console.log('data', data);

    // plans
    // use this component to check for ping messages
    // pass new block hight to each child components to trigger reload
    // - leaderboard (listened to game ping messages)
    // - game (board, listened to player ping messages)
    // - player score will update leaderboard score too
    const isMultiplayer = boardId.includes('-');
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        {#if isMultiplayer}
            <Brand />
            <Leaderboard
                currentRound={data?.currentRound}
                gameLeaderboard={data?.gameLeaderboard}
                roundLeaderboard={data?.roundLeaderboard?.[0]}
            />
        {:else}
            <UserSidebar bind:username />
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
                    playerChainId={playerChainId}
                    boardId={boardId}
                    canStartNewGame={!isMultiplayer}
                    showBestScore={!isMultiplayer}
                    canMakeMove={player === username}
                />
            </div>
        </div>
    </svelte:fragment>
</MainTemplate>
