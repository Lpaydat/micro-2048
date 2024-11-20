<script lang="ts">
  import { queryStore, subscriptionStore, gql, getContextClient } from '@urql/svelte';

  import BoardHeader from "../molecules/BoardHeader.svelte";
  import Board from '../molecules/Board.svelte';
	import { makeMove } from '$lib/graphql/mutations/makeMove';
	import { onDestroy, onMount } from 'svelte';
  import { hashesStore, isHashesListVisible } from '$lib/stores/hashesStore';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { clearMessages } from '$lib/graphql/mutations/clearMessages';
	import { applicationId, port } from '$lib/constants';

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

  let specBoardId = $page.url.searchParams.get('boardId');
  let localBoardId: string | null = null;
  let gameBoardId: string | undefined = boardId;

  onMount(() => {
    localBoardId = localStorage.getItem('boardId');
    if (!isMultiplayer && localBoardId && boardId === undefined) {
      gameBoardId = specBoardId || localBoardId;
      if (specBoardId) {
        canMakeMove = false;
      } else {
        canMakeMove = true;
      }
    }
  });

  // Update gameBoardId when boardId prop changes
  $: if (boardId !== undefined) {
    gameBoardId = boardId;
  }

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
    variables: { boardId: gameBoardId },
    requestPolicy: 'network-only',
  });
  $: score = $game.data?.board?.score || 0;

  $: if (isMultiplayer && $game.data?.board === null) {
    goto('/error');
  }

  let moveTimeout: NodeJS.Timeout | null = null;
  let keyPressTime: number | null = null;
  let pingTime: number | null = null;

  // Add a new store for tracking moves
  let moveStartTimes: Record<string, number> = {};

  // Mutation functions
  const makeMoveMutation = ({ boardId, direction }: { boardId: string, direction: string }) => {
    if (!canMakeMove) return;

    canMakeMove = false;
    moveStartTimes[direction] = Date.now(); // Store the start time for this move

    // Set a timeout to re-enable moves after 200ms
    moveTimeout = setTimeout(() => {
      canMakeMove = true;
    }, 100);

    makeMove(client, boardId, direction);
    // clearMessages(playerChainId, applicationId, port);
  };

  // Subscription for notifications
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
      clearTimeout(moveTimeout);
    }
    // Calculate ping time when new block arrives
    const lastMove = Object.entries(moveStartTimes)[0];
    if (lastMove) {
      const [direction, startTime] = lastMove;
      pingTime = Date.now() - startTime;
      delete moveStartTimes[direction]; // Clean up the stored time
    }
    game.reexecute({ requestPolicy: 'network-only' });
  }

  $: rendered = false;
  $: if (!$game.fetching && $game.data?.board) {
    rendered = true;
  }

  // Logs for move history
  let lastHash = '';
  $: if (
    $playerMessages?.data?.notifications?.reason?.NewBlock?.hash
    && lastHash !== $playerMessages?.data?.notifications?.reason?.NewBlock?.hash
  ) {
    lastHash = $playerMessages?.data?.notifications?.reason?.NewBlock?.hash;
    if (lastHash) {
      hashesStore.update(logs => [{ hash: lastHash, timestamp: new Date().toISOString() }, ...logs]);
    }
  }

  // Utility functions
  const hasWon = (board: number[][]) => board.some(row => row.includes(11));

  // Add touch handling variables
  let touchStartX: number | null = null;
  let touchStartY: number | null = null;
  const SWIPE_THRESHOLD = 50; // minimum distance for a swipe

  // Add touch event handlers
  const handleTouchStart = (event: TouchEvent) => {
    // Only prevent default if touch started on game board
    if (event.target instanceof Element && event.target.closest('.game-board')) {
      event.preventDefault();
    }
    touchStartX = event.touches[0].clientX;
    touchStartY = event.touches[0].clientY;
  };

  const handleTouchEnd = (event: TouchEvent) => {
    // Only prevent default if touch ended on game board
    if (event.target instanceof Element && event.target.closest('.game-board')) {
      event.preventDefault();
    }
    if (!touchStartX || !touchStartY || $game.data?.board?.isEnded || !gameBoardId) return;

    const touchEndX = event.changedTouches[0].clientX;
    const touchEndY = event.changedTouches[0].clientY;

    const deltaX = touchEndX - touchStartX;
    const deltaY = touchEndY - touchStartY;

    // Determine if the swipe was primarily horizontal or vertical
    if (Math.abs(deltaX) > Math.abs(deltaY)) {
      // Horizontal swipe
      if (Math.abs(deltaX) >= SWIPE_THRESHOLD) {
        if (deltaX > 0) {
          makeMoveMutation({ boardId: gameBoardId, direction: 'ArrowRight' });
        } else {
          makeMoveMutation({ boardId: gameBoardId, direction: 'ArrowLeft' });
        }
      }
    } else {
      // Vertical swipe
      if (Math.abs(deltaY) >= SWIPE_THRESHOLD) {
        if (deltaY > 0) {
          makeMoveMutation({ boardId: gameBoardId, direction: 'ArrowDown' });
        } else {
          makeMoveMutation({ boardId: gameBoardId, direction: 'ArrowUp' });
        }
      }
    }

    // Reset touch coordinates
    touchStartX = null;
    touchStartY = null;
  };

  // Add new handler for touch move to prevent scrolling during swipe
  const handleTouchMove = (event: TouchEvent) => {
    // Only prevent default if touch move is on game board
    if (event.target instanceof Element && event.target.closest('.game-board')) {
      event.preventDefault();
    }
  };

  // Update the existing handleKeydown function to handle arrow keys
  const handleKeydown = (event: KeyboardEvent) => {
    if ($game.data?.board?.isEnded || !gameBoardId) return;
    keyPressTime = Date.now();
    
    // Map arrow keys to directions
    const validKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'];
    if (validKeys.includes(event.key)) {
      makeMoveMutation({ boardId: gameBoardId, direction: event.key });
    }
  };

  const getOverlayMessage = (board: number[][]) => {
    if (!isMultiplayer) {
      return hasWon(board) ? "Congratulations! You Won!" : "Game Over! You Lost!";
    }
    return "Game Over!";
  };

  // Add new prop for board size
  let boardSize: 'sm' | 'md' | 'lg' = 'lg';

  function updateBoardSize() {
      if (window.innerWidth < 480) {
          boardSize = 'sm';
      } else if (window.innerWidth < 1248) {
          boardSize = 'md';
      } else {
          boardSize = 'lg';
      }
  }

  onMount(() => {
      updateBoardSize();
      window.addEventListener('resize', updateBoardSize);
      return () => window.removeEventListener('resize', updateBoardSize);
  });
</script>

<svelte:window 
  on:keydown={handleKeydown}
/>

<div 
  class="game-container {boardSize}"
>
  <BoardHeader bind:boardId={gameBoardId} {canStartNewGame} {showBestScore} {player} value={score} size={boardSize} />
  {#if rendered}
    <div 
      class="game-board"
      on:touchstart={handleTouchStart}
      on:touchmove={handleTouchMove}
      on:touchend={handleTouchEnd}
    >
      <Board board={$game.data?.board?.board} size={boardSize} />
      {#if $game.data?.board?.isEnded || isEnded}
        <div class="overlay">
          <p>{getOverlayMessage($game.data?.board?.board)}</p>
        </div>
      {/if}
    </div>
    <div class="mt-2 flex items-center justify-center gap-4 text-sm">
      <button 
        class="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-surface-800/50 hover:bg-black/50 transition-colors"
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
    <Board board={[[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]} size={boardSize} />
  {/if}
</div>

<style>
  .game-container {
    margin: 0 auto;
    text-align: center;
    overflow: visible;
  }

  .game-container.lg {
    max-width: 555px;
  }

  .game-container.md {
    max-width: 460px;
  }

  .game-container.sm {
    max-width: 370px;
  }

  .game-board {
    position: relative;
    width: 100%;
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

  .game-container.sm .overlay {
    font-size: 1.2em;
  }

  .game-container.md .overlay {
    font-size: 1.35em;
  }

  .game-container.lg .overlay {
    font-size: 1.5em;
  }
</style>
