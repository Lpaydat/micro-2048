<script lang="ts">
  import { queryStore, subscriptionStore, gql, getContextClient } from '@urql/svelte';

  import BoardHeader from "../molecules/BoardHeader.svelte";
	import { makeMove } from '$lib/graphql/mutations/makeMove';
	import { onDestroy, onMount } from 'svelte';
  import { hashesStore, isHashesListVisible } from '$lib/stores/hashesStore';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { clearMessages } from '$lib/graphql/mutations/clearMessages';
	import { applicationId, port } from '$lib/constants';
	import { genInitialState as createState } from '$lib/game/game';
	import Tablet from '../molecules/Tablet.svelte';
	import type { GameKeys, GameState } from '$lib/game/models';
	import { isNewGameCreated, setGameCreationStatus } from '$lib/stores/gameStore';
	import { boardToString } from '$lib/game/utils';

  // Props
  export let isMultiplayer: boolean = false;
  export let isEnded: boolean = false;
  export let player: string;
  export let score: number = 0;
  export let playerChainId: string;
  export let boardId: string | undefined = undefined;
  export let canStartNewGame: boolean = true;
  export let canMakeMove: boolean = true;
  export let showBestScore: boolean = true;

  // Board ID Management
  let specBoardId = $page.url.searchParams.get('boardId');
  let localBoardId: string | null = null;
  let gameBoardId: string | undefined = boardId;

  // GraphQL Definitions
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

  // State Management
  const client = getContextClient();
  let state: GameState | undefined;
  let isInitialized = false;
  let rendered = false;
  let blockHeight = 0;
  let lastHash = '';
  let moveStartTimes: Record<string, number> = {};
  let isSynced: boolean = false;

  // Timers and Flags
  let moveTimeout: NodeJS.Timeout | null = null;
  let syncTimeout: NodeJS.Timeout | null = null;
  let keyPressTime: number | null = null;
  let pingTime: number | null = null;
  $: shouldSyncGame = false;

  // Responsive Design
  let boardSize: 'sm' | 'md' | 'lg' = 'lg';
  
  // Touch Handling
  let touchStartX: number | null = null;
  let touchStartY: number | null = null;
  const SWIPE_THRESHOLD = 50;

  // GraphQL Queries and Subscriptions
  $: game = queryStore({
    client,
    query: GET_BOARD_STATE,
    variables: { boardId: gameBoardId },
    requestPolicy: 'network-only',
  });

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

  // Reactive Statements
  $: score = $game.data?.board?.score || 0;
  
  $: if (isMultiplayer && $game.data?.board === null) {
    goto('/error');
  }

  $: if (!$game.fetching && $game.data?.board) {
    rendered = true;
  }

  $: if (boardId !== undefined) {
    gameBoardId = boardId;
    setGameCreationStatus(true);
  }

  $: bh = $playerMessages?.data?.notifications?.reason?.NewBlock?.height;
  $: if (bh && bh !== blockHeight) {
    handleNewBlock(bh);
  }

  $: if (
    $playerMessages?.data?.notifications?.reason?.NewBlock?.hash
    && lastHash !== $playerMessages?.data?.notifications?.reason?.NewBlock?.hash
  ) {
    handleNewHash($playerMessages?.data?.notifications?.reason?.NewBlock?.hash);
  }

  $: if (
    $game.data?.board &&
    gameBoardId &&
    player &&
    (!isInitialized || $isNewGameCreated || $game.data?.board?.isEnded || shouldSyncGame)
  ) {
    handleGameStateUpdate();
  }
  

  // Utility Functions
  const hasWon = (board: number[][]) => board.some(row => row.some(cell => cell >= 11));

  const getOverlayMessage = (board: number[][]) => {
    if (!isMultiplayer) {
      return hasWon(board) ? "Congratulations! You Won!" : "Game Over! You Lost!";
    }
    return "Game Over!";
  };

  // Game State Handlers
  function handleNewBlock(newBlockHeight: number) {
    blockHeight = newBlockHeight;
    canMakeMove = true;
    if (moveTimeout) clearTimeout(moveTimeout);
    
    const lastMove = Object.entries(moveStartTimes)[0];
    if (lastMove) {
      const [direction, startTime] = lastMove;
      pingTime = Date.now() - startTime;
      delete moveStartTimes[direction];
    }
    game.reexecute({ requestPolicy: 'network-only' });
  }

  function handleNewHash(hash: string) {
    lastHash = hash;
    if (lastHash) {
      hashesStore.update(logs => [{ hash: lastHash, timestamp: new Date().toISOString() }, ...logs]);
    }
  }

  function handleGameStateUpdate() {
    if (!gameBoardId) return;
    state = createState($game.data?.board?.board, 4, gameBoardId, player);
    isInitialized = true;
    shouldSyncGame = false;
    isSynced = true;
    setGameCreationStatus(false);
  }

  // Movement Functions
  const makeMoveMutation = (
    { boardId, direction, timestamp }: { boardId: string, direction: string, timestamp: string }
  ) => {
    makeMove(client, boardId, direction, timestamp);
    clearMessages(playerChainId, applicationId, port);
  };

  const move = async (boardId: string, direction: GameKeys) => {
    if (!canMakeMove || $game.data?.board?.isEnded) return;

    canMakeMove = false;
    shouldSyncGame = false;
    isSynced = false;
    moveStartTimes[direction] = Date.now();

    moveTimeout = setTimeout(() => { canMakeMove = true; }, 100);
    if (syncTimeout) clearTimeout(syncTimeout);
    syncTimeout = setTimeout(() => { shouldSyncGame = true; }, 2000);

    const timestamp = Date.now().toString();
    makeMoveMutation({ boardId, direction, timestamp });
    const prevTablet = boardToString(state?.tablet);
    state = await state?.actions[direction](state, timestamp, prevTablet);
  };

  // Event Handlers
  const handleTouchStart = (event: TouchEvent) => {
    if (event.target instanceof Element && event.target.closest('.game-board')) {
      event.preventDefault();
    }
    touchStartX = event.touches[0].clientX;
    touchStartY = event.touches[0].clientY;
  };

  const handleTouchMove = (event: TouchEvent) => {
    if (event.target instanceof Element && event.target.closest('.game-board')) {
      event.preventDefault();
    }
  };

  const handleTouchEnd = async (event: TouchEvent) => {
    if (event.target instanceof Element && event.target.closest('.game-board')) {
      event.preventDefault();
    }
    if (!touchStartX || !touchStartY || $game.data?.board?.isEnded || !gameBoardId) return;

    const touchEndX = event.changedTouches[0].clientX;
    const touchEndY = event.changedTouches[0].clientY;

    const deltaX = touchEndX - touchStartX;
    const deltaY = touchEndY - touchStartY;

    if (Math.abs(deltaX) > Math.abs(deltaY)) {
      if (Math.abs(deltaX) >= SWIPE_THRESHOLD) {
        if (deltaX > 0) {
          move(gameBoardId, 'ArrowRight');
        } else {
          move(gameBoardId, 'ArrowLeft');
        }
      }
    } else {
      if (Math.abs(deltaY) >= SWIPE_THRESHOLD) {
        if (deltaY > 0) {
          move(gameBoardId, 'ArrowDown');
        } else {
          move(gameBoardId, 'ArrowUp');
        }
      }
    }

    touchStartX = null;
    touchStartY = null;
  };

  const handleKeydown = async (event: KeyboardEvent) => {
    if ($game.data?.board?.isEnded || !gameBoardId) return;
    keyPressTime = Date.now();
    
    const validKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'];
    if (validKeys.includes(event.key)) {
      move(gameBoardId, event.key as GameKeys);
    }
  };

  // Responsive Design Functions
  const updateBoardSize = () => {
    if (window.innerWidth < 480) boardSize = 'sm';
    else if (window.innerWidth < 1024) boardSize = 'md';
    else boardSize = 'lg';
  };

  // Lifecycle Hooks
  onMount(() => {
    localBoardId = localStorage.getItem('boardId');
    if (!isMultiplayer && localBoardId && boardId === undefined) {
      gameBoardId = specBoardId || localBoardId;
      canMakeMove = !specBoardId;
    }

    updateBoardSize();
    window.addEventListener('resize', updateBoardSize);
    return () => window.removeEventListener('resize', updateBoardSize);
  });

  onDestroy(() => {
    if (playerMessages) {
      playerMessages.pause();
      hashesStore.set([]);
    }
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
      {#if state}
        <Tablet tablet={state.tablet} size={boardSize} />
      {/if}
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
        <span class="text-surface-400">|</span>
        <span class={isSynced ? "text-emerald-400" : "text-yellow-400"}>
          {isSynced ? "synced" : "syncing"}
        </span>
      </button>
    </div>
  {:else}
    <Tablet size={boardSize} />
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
