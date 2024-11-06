<script lang="ts">
  import { queryStore, mutationStore, subscriptionStore, gql, getContextClient } from '@urql/svelte';
  import { onMount } from "svelte";

  import { getSubscriptionId } from '$lib/getSubscriptionId';
  import Header from "../molecules/BoardHeader.svelte";
  import Board from '../molecules/Board.svelte';

  export let player: string;
  export let playerChainId: string;
  export let gameChainId: string | undefined;
  export let boardId: string | undefined = undefined;

  export let canStartNewGame: boolean = true;
  export let canMakeMove: boolean = true;
  export let showBestScore: boolean = true;

  // accept chainId for subscription
  // boardId if it's multiplayer
  // boardId for queries
  // player name for queries

  // use combination of playerChainId and gameChainId for subscription
  // playerChainId used for game board queries
  // gameChainId used for game state queries

  // GraphQL queries, mutations, and subscriptions
  const GET_BOARD_STATE = gql`
    query BoardState($boardId: Int!) {
      board(boardId: $boardId) {
        boardId
        board
        score
        isEnded
      }
    }
  `;

  const MAKE_MOVE = gql`
    mutation MakeMove($boardId: ID!, $direction: String!) {
      makeMove(boardId: $boardId, direction: $direction)
    }
  `;

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

  // Initialize client and game state
  let client = getContextClient();

  // Reactive statement for game state
  $: game = queryStore({
    client,
    query: GET_BOARD_STATE,
    variables: { boardId },
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
  const makeMoveMutation = ({ boardId, direction }: { boardId: string, direction: string }) => {
    if (!canMakeMove) return;

    const formattedDirection = direction.replace('Arrow', '');
    if (!Object.values(directionList).includes(formattedDirection)) {
      console.error('Invalid direction:', direction);
      return;
    }
    mutationStore({
      client,
      query: MAKE_MOVE,
      variables: { boardId, direction: formattedDirection },
    });
  };

  // Subscription for notifications
  const gameMessages = subscriptionStore({
    client,
    query: GAME_PING_SUBSCRIPTION,
    variables: { chainId: gameChainId },
  });

  const playerMessages = subscriptionStore({
    client,
    query: PLAYER_PING_SUBSCRIPTION,
    variables: { chainId: playerChainId },
  });

  // Reactive statements for block height and rendering
  let blockHeight = 0;
  $: bh = $gameMessages.data?.notifications?.reason?.NewBlock?.height;
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
  $: isCurrentGame = $game.data?.game?.boardId === boardId;
  $: if ($gameMessages.data?.notifications?.reason?.NewBlock?.hash
    && lastHash !== $gameMessages.data.notifications.reason.NewBlock.hash
    && isCurrentGame)
  {
    lastHash = $gameMessages.data.notifications.reason.NewBlock.hash;
    logs = [{ hash: lastHash, timestamp: new Date().toISOString() }, ...logs];
  }

  // Utility functions
  const hasWon = (board: number[][]) => board.some(row => row.includes(11));

  const handleKeydown = (event: KeyboardEvent) => {
    if ($game.data?.game?.isEnded || !boardId) return;
    makeMoveMutation({ boardId, direction: event.key });
  };

  const getOverlayMessage = (board: number[][]) => hasWon(board) ? "Congratulations! You Won!" : "Game Over! You Lost!";
</script>

<svelte:window on:keydown={handleKeydown} />


<div class="game-container">
  <Header {canStartNewGame} {showBestScore} {player} value={$game.data?.game?.score || 0} />
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
