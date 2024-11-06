<script lang="ts">
  import { queryStore, mutationStore, subscriptionStore, gql, getContextClient } from '@urql/svelte';
  import { onMount } from "svelte";

  import { getSubscriptionId } from '$lib/getSubscriptionId';
  import Header from "../molecules/BoardHeader.svelte";
  import Board from './Board.svelte';

  export let canStartNewGame: boolean = true;
  export let canMakeMove: boolean = true;

  // accept chainId for subscription
  // gameId if it's multiplayer
  // boardId for queries
  // player name for queries

  // use combination of playerChainId and gameChainId for subscription
  // playerChainId used for game board queries
  // gameChainId used for game state queries

  // GraphQL queries, mutations, and subscriptions
  const GET_GAME_STATE = gql`
    query GetGameState($gameId: Int!) {
      game(gameId: $gameId) {
        gameId
        board
        score
        isEnded
      }
    }
  `;

  const NEW_BOARD = gql`
    mutation NewBoard($seed: Int!, $subscriptionId: String!) {
      newBoard(seed: $seed, subscriptionId: $subscriptionId)
    }
  `;

  const MAKE_MOVE = gql`
    mutation MakeMove($gameId: ID!, $direction: String!, $subscriptionId: String!) {
      makeMove(gameId: $gameId, direction: $direction, subscriptionId: $subscriptionId)
    }
  `;

  const NOTIFICATION_SUBSCRIPTION = gql`
    subscription Notifications($chainId: ID!) {
      notifications(chainId: $chainId)
    }
  `;

  // Initialize client and game state
  let client = getContextClient();
  let gameId = 0;
  let subscriptionId = getSubscriptionId();

  // Reactive statement for game state
  $: game = queryStore({
    client,
    query: GET_GAME_STATE,
    variables: { gameId },
    requestPolicy: 'network-only',
  });

  // Enum for move directions
  const directionList = {
    Up: "Up",
    Down: "Down",
    Left: "Left",
    Right: "Right"
  }

  // Mutation functions
  const newGameMutation = ({ seed }: { seed: number }) => {
    mutationStore({
      client,
      query: NEW_BOARD,
      variables: { seed, subscriptionId },
    });
  };

  const makeMoveMutation = ({ gameId, direction }: { gameId: number, direction: string }) => {
    if (!canMakeMove) return;

    const formattedDirection = direction.replace('Arrow', '');
    if (!Object.values(directionList).includes(formattedDirection)) {
      console.error('Invalid direction:', direction);
      return;
    }
    mutationStore({
      client,
      query: MAKE_MOVE,
      variables: { gameId, direction: formattedDirection, subscriptionId },
    });
  };

  // Subscription for notifications
  const messages = subscriptionStore({
    client,
    query: NOTIFICATION_SUBSCRIPTION,
    variables: { chainId: subscriptionId },
  });

  // Game initialization and lifecycle
  const newGame = () => {
    gameId = Math.floor(Math.random() * 65536) + 1;
    logs = []
    newGameMutation({ seed: gameId });
  };

  onMount(() => {
    setTimeout(() => {
      newGame();
    }, 50);
  });

  // Reactive statements for block height and rendering
  let blockHeight = 0;
  $: bh = $messages.data?.notifications?.reason?.NewBlock?.height;
  $: if (bh && bh !== blockHeight) {
    blockHeight = bh;
    game.reexecute({ requestPolicy: 'network-only' });
  }

  $: rendered = false;
  $: if (!$game.fetching) {
    rendered = true;
  }

  // Logs for move history
  let logs: { hash: string, timestamp: string }[] = [];
  let lastHash = '';
  $: isCurrentGame = $game.data?.game?.gameId === gameId;
  $: if ($messages.data?.notifications?.reason?.NewBlock?.hash
    && lastHash !== $messages.data.notifications.reason.NewBlock.hash
    && isCurrentGame)
  {
    lastHash = $messages.data.notifications.reason.NewBlock.hash;
    logs = [{ hash: lastHash, timestamp: new Date().toISOString() }, ...logs];
  }

  // Utility functions
  const hasWon = (board: number[][]) => board.some(row => row.includes(11));

  const handleKeydown = (event: KeyboardEvent) => {
    if ($game.data?.game?.isEnded) return;
    makeMoveMutation({ gameId, direction: event.key });
  };

  const getOverlayMessage = (board: number[][]) => hasWon(board) ? "Congratulations! You Won!" : "Game Over! You Lost!";
</script>

<svelte:window on:keydown={handleKeydown} />


<div class="game-container">
  <Header {canStartNewGame} value={$game.data?.game?.score || 0} on:click={newGame} />
  {#if $game.data?.game}
    <div class="game-board">
      <Board board={$game.data?.game?.board} />
      {#if $game.data?.game?.isEnded}
        <div class="overlay">
          <p>{getOverlayMessage($game.data?.game?.board)}</p>
        </div>
      {/if}
    </div>
  {:else}
    <Board board={[[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]} />
  {/if}
</div>

<style>
  .game-container {
    max-width: 554px;
    background-color: transparent;;
    margin: 0 auto;
    text-align: center;
    overflow: visible;
  }

  .game-board {
    position: relative;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
  }

  .overlay {
    position: absolute;
    font-weight: bold;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.6);
    border-radius: 6px;
    color: white;
    display: flex;
    justify-content: center;
    align-items: center;
    font-size: 1.5em;
  }
</style>
