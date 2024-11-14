<script lang="ts">
  import { queryStore, subscriptionStore, gql, getContextClient } from '@urql/svelte';

  import BoardHeader from "../molecules/BoardHeader.svelte";
  import Board from '../molecules/Board.svelte';
	import { makeMove } from '$lib/graphql/mutations/makeMove';
	import { onDestroy } from 'svelte';
  import { hashesStore, isHashesListVisible } from '$lib/stores/hashesStore';

  export let isMultiplayer: boolean = false;
  export let isEnded: boolean = false;
  export let player: string;
  export let score: number = 0;
  export let playerChainId: string;
  export let boardId: string | undefined = undefined;

  export let canStartNewGame: boolean = true;
  export let canMakeMove: boolean = true;
  export let showBestScore: boolean = true;

  // TODO: currently, game is slow because it need to wait for cross-chain messages to be processed

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

  const PLAYER_PING_SUBSCRIPTION = gql`
    subscription Notifications($chainId: ID!) {
      notifications(chainId: $chainId)
    }
  `;

  // Initialize client and game state
  const client = getContextClient();

  // Reactive statement for game state
  $: game = queryStore({
    client,
    query: GET_BOARD_STATE,
    variables: { boardId },
    requestPolicy: 'network-only',
  });
  $: score = $game.data?.board?.score || 0;

  let moveTimeout: NodeJS.Timeout | null = null;
  let keyPressTime: number | null = null;
  let pingTime: number | null = null;

  // Mutation functions
  const makeMoveMutation = ({ boardId, direction }: { boardId: string, direction: string }) => {
    if (!canMakeMove) return;

    canMakeMove = false;

    // Set a timeout to re-enable moves after 200ms
    moveTimeout = setTimeout(() => {
      canMakeMove = true;
    }, 100);

    makeMove(client, boardId, direction);
  };

  // Subscription for notifications
  // const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
  let playerMessages: any;
  $: {
    if (playerChainId) {
      playerMessages = subscriptionStore({
        client,
        query: PLAYER_PING_SUBSCRIPTION,
        variables: { chainId: playerChainId },
      });
    }
  }

  onDestroy(() => {
    if (playerMessages) {
      playerMessages.pause();
      hashesStore.set([]);
    }
  });

  // Reactive statements for block height and rendering
  let blockHeight = 0;
  $: bh = $playerMessages?.data?.notifications?.reason?.NewBlock?.height;
  $: if (bh && bh !== blockHeight) {
    blockHeight = bh;
    canMakeMove = true;
    if (moveTimeout) {
      clearTimeout(moveTimeout); // Clear the timeout if a new block height is received
    }
    game.reexecute({ requestPolicy: 'network-only' });
  }

  $: rendered = false;
  $: if (!$game.fetching && $game.data?.board) {
    rendered = true;
  }

  $: if (keyPressTime && !$game.fetching) {
    pingTime = Date.now() - keyPressTime; // Calculate the ping time
    keyPressTime = null; // Reset keyPressTime
  }

  // Logs for move history
  let lastHash = '';
  $: if (
    $playerMessages?.data?.notifications?.reason?.NewBlock?.hash
    && lastHash !== $playerMessages?.data?.notifications?.reason?.NewBlock?.hash
  ) {
    lastHash = $playerMessages?.data?.notifications?.reason?.NewBlock?.hash;
    console.log('lastHash', lastHash);
    if (lastHash) {
      hashesStore.update(logs => [{ hash: lastHash, timestamp: new Date().toISOString() }, ...logs]);
    }
  }

  // Utility functions
  const hasWon = (board: number[][]) => board.some(row => row.includes(11));

  const handleKeydown = (event: KeyboardEvent) => {
    if ($game.data?.board?.isEnded || !boardId) return;
    keyPressTime = Date.now(); // Capture the time when a key is pressed
    makeMoveMutation({ boardId, direction: event.key });
  };

  const getOverlayMessage = (board: number[][]) => {
    if (!isMultiplayer) {
      return hasWon(board) ? "Congratulations! You Won!" : "Game Over! You Lost!";
    }
    return "Game Over!";
  };
</script>

<svelte:window on:keydown={handleKeydown} />


<div class="game-container">
  <BoardHeader bind:boardId={boardId} {canStartNewGame} {showBestScore} {player} value={score} />
  {#if rendered}
    <div class="game-board">
      <Board board={$game.data?.board?.board} />
      {#if $game.data?.board?.isEnded || isEnded}
        <div class="overlay">
          <p>{getOverlayMessage($game.data?.board?.board)}</p>
        </div>
      {/if}
    </div>
    <div class="mt-2 flex items-center justify-center gap-4 text-sm">
      <button 
        class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-surface-700/50 hover:bg-surface-600/50 transition-colors"
        on:click={() => isHashesListVisible.update(current => !current)}
      >
        <span 
          class="font-mono text-emerald-400 cursor-pointer" 
          title={lastHash || "No hash available"}
        >
          {#if lastHash}
            {lastHash.slice(0, 6)}...{lastHash.slice(-4)}
          {:else}
            ---
          {/if}
        </span>
        <span class="text-surface-400">|</span>
        <span class="text-orange-400">{pingTime || 0}<span class="text-surface-400 text-xs ml-1">ms</span></span>
      </button>
    </div>
  {:else}
    <Board board={[[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]} />
  {/if}
</div>

<style>
  .game-container {
    max-width: 555px;
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
