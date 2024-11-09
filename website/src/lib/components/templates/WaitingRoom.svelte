<script lang="ts">
	import { getContextClient, gql, subscriptionStore } from '@urql/svelte';
	import { onDestroy } from 'svelte';
    import Clock from 'lucide-svelte/icons/clock-4';
    import { page } from '$app/stores';

	import ActionButton from '../atoms/ActionButton.svelte';
	import PageHeader from "../molecules/PageHeader.svelte";
    import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte'
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import WaitingPlayers from "../organisms/WaitingPlayers.svelte";
	import UserSidebar from "../organisms/UserSidebar.svelte";
	import { joinGame } from '$lib/graphql/mutations/joinGame';
	import { leaveGame } from '$lib/graphql/mutations/leaveGame';
	import { startGame } from '$lib/graphql/mutations/startGame';
	import { endGame } from '$lib/graphql/mutations/endGame';
	import { goto } from '$app/navigation';
	import { getGameDetails } from '$lib/graphql/queries/getGameDetails';
	import { userStore } from '$lib/stores/userStore';

    const gameId = $page.params.gameId;

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
        variables: { chainId: gameId },
    });

    $: game = getGameDetails(client, gameId);
    $: data = $game.data?.eliminationGame && !$game.fetching 
        ? {
            ...$game.data.eliminationGame,
            playerCount: $game.data.eliminationGame.players.length
        }
        : undefined;

    $: if (data?.status === 'Active' && isJoined) {
        const playerBoardId = `${gameId}-${data.currentRound}-${username}`;
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
    $: bh = $gameMessages.data?.notifications?.reason?.NewBlock?.height;
    $: if (bh && bh !== blockHeight) {
        blockHeight = bh;
        game.reexecute({ requestPolicy: 'network-only' });
    }

    onDestroy(() => {
        gameMessages.pause();
    })

    const handleJoinGame = () => {
        joinGame(client, username, gameId);
        game.reexecute({ requestPolicy: 'network-only' });
    }

    const handleLeaveGame = () => {
        leaveGame(client, username, gameId);
    }

    const handleStartGame = () => {
        if (data?.playerCount < 2) return;
        startGame(client, gameId, username);
    }

    const handleEndGame = () => {
        endGame(client, gameId, username);
    }
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <UserSidebar bind:username />
    </svelte:fragment>

    <svelte:fragment slot="main">
        <PageHeader title={gameName} prevPage={data?.status === 'Ended' ? '/elimination': undefined}>
            <svelte:fragment slot="icon">
                <Clock size={28} />
            </svelte:fragment>
            <svelte:fragment slot="actions">
                {#if isLoaded && $userStore.username && data?.status === 'Waiting'}
                    {#if isHost}
                        <ActionButton label="START GAME" on:click={handleStartGame} disabled={data.playerCount < 2} />
                        <ActionButton label="END GAME" hoverColor="danger" on:click={handleEndGame} />
                    {:else if canJoinGame}
                        <ActionButton label="JOIN GAME" on:click={handleJoinGame} />
                    {:else if isJoined}
                        <ActionButton label="LEAVE GAME" on:click={handleLeaveGame} />
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
