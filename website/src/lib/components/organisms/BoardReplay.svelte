<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { getBoardMoveHistory } from '$lib/graphql/queries/getBoardMoveHistory';
	import { getClient } from '$lib/client';
	import Board from './Board.svelte';
	import { boardSize } from '$lib/stores/gameStore';

	interface Props {
		boardId: string;
		chainId: string;
	}

	let { boardId, chainId }: Props = $props();

	const client = $derived(getClient(chainId));
	const moveHistory = $derived(getBoardMoveHistory(client, boardId));

	let currentMoveIndex = $state(0);
	let isPlaying = $state(false);
	let playbackSpeed = $state(1); // 0.5x, 1x, 2x, 4x
	let playbackInterval: NodeJS.Timeout | null = null;

	const totalMoves = $derived($moveHistory?.data?.boardMoveHistory?.totalMoves ?? 0);
	const moves = $derived($moveHistory?.data?.boardMoveHistory?.moves ?? []);
	const player = $derived($moveHistory?.data?.boardMoveHistory?.player ?? '');

	const currentBoard = $derived.by(() => {
		if (currentMoveIndex === 0 || moves.length === 0) {
			return null;
		}
		return moves[currentMoveIndex - 1]?.boardAfter ?? null;
	});

	const currentScore = $derived.by(() => {
		if (currentMoveIndex === 0 || moves.length === 0) {
			return 0;
		}
		return moves[currentMoveIndex - 1]?.scoreAfter ?? 0;
	});

	const currentDirection = $derived.by(() => {
		if (currentMoveIndex === 0 || moves.length === 0) {
			return null;
		}
		return moves[currentMoveIndex - 1]?.direction ?? null;
	});

	const play = () => {
		if (currentMoveIndex >= totalMoves) {
			currentMoveIndex = 0;
		}
		isPlaying = true;
		startPlayback();
	};

	const pause = () => {
		isPlaying = false;
		if (playbackInterval) {
			clearInterval(playbackInterval);
			playbackInterval = null;
		}
	};

	const restart = () => {
		currentMoveIndex = 0;
		pause();
	};

	const nextMove = () => {
		if (currentMoveIndex < totalMoves) {
			currentMoveIndex++;
		}
		if (currentMoveIndex >= totalMoves) {
			pause();
		}
	};

	const prevMove = () => {
		if (currentMoveIndex > 0) {
			currentMoveIndex--;
		}
	};

	const setSpeed = (speed: number) => {
		playbackSpeed = speed;
		if (isPlaying) {
			pause();
			play();
		}
	};

	const startPlayback = () => {
		if (playbackInterval) {
			clearInterval(playbackInterval);
		}
		
		const baseDelay = 500; // 500ms at 1x speed
		const delay = baseDelay / playbackSpeed;
		
		playbackInterval = setInterval(() => {
			if (currentMoveIndex < totalMoves) {
				currentMoveIndex++;
			} else {
				pause();
			}
		}, delay);
	};

	const handleSliderChange = (e: Event) => {
		const target = e.target as HTMLInputElement;
		currentMoveIndex = parseInt(target.value);
		if (isPlaying) {
			pause();
		}
	};

	onDestroy(() => {
		if (playbackInterval) {
			clearInterval(playbackInterval);
		}
	});

	const sizeConfig = {
		xs: { width: 296, fontSize: 'text-sm', scoreSize: 'text-md' },
		sm: { width: 370, fontSize: 'text-base', scoreSize: 'text-lg' },
		md: { width: 460, fontSize: 'text-xl', scoreSize: 'text-2xl' },
		lg: { width: 555, fontSize: 'text-2xl', scoreSize: 'text-2xl' }
	};

	const currentSize = $derived(sizeConfig[$boardSize]);
</script>

<div class="replay-container {$boardSize}">
	{#if $moveHistory.fetching}
		<div class="flex items-center justify-center p-8">
			<div class="text-surface-400">Loading replay data...</div>
		</div>
	{:else if $moveHistory.error}
		<div class="flex items-center justify-center p-8">
			<div class="text-red-400">Error loading replay: {$moveHistory.error.message}</div>
		</div>
	{:else if !$moveHistory.data?.boardMoveHistory}
		<div class="flex items-center justify-center p-8">
			<div class="text-surface-400">No replay data available</div>
		</div>
	{:else}
		<!-- Header with player and score -->
		<div class="mb-4 flex items-center justify-between" style="width: {currentSize.width}px">
			<div class="flex items-center gap-2">
				<span class="text-surface-400">Player:</span>
				<span class="{currentSize.fontSize} font-bold text-white">{player}</span>
			</div>
			<div class="flex min-w-16 flex-col rounded-md bg-[#bbada0] p-2 font-bold text-white">
				<div class="text-xs text-[#eee4da]">Score</div>
				<div class="{currentSize.scoreSize} text-center">{currentScore}</div>
			</div>
		</div>

		<!-- Board Display -->
		<Board tablet={currentBoard} canMakeMove={false} isEnded={false} moveCallback={() => {}}>
			{#snippet header()}
				<div class="mb-2 flex items-center justify-between gap-2">
					<div class="text-surface-400">
						{#if currentDirection}
							Last move: <span class="font-bold text-orange-400">{currentDirection}</span>
						{/if}
					</div>
					<div class="text-surface-400">
						Move <span class="font-bold text-purple-400">{currentMoveIndex}</span> / {totalMoves}
					</div>
				</div>
			{/snippet}
		</Board>

		<!-- Playback Controls -->
		<div class="mt-4 flex flex-col gap-4" style="width: {currentSize.width}px">
			<!-- Progress Slider -->
			<input
				type="range"
				min="0"
				max={totalMoves}
				bind:value={currentMoveIndex}
				onchange={handleSliderChange}
				class="w-full"
			/>

			<!-- Control Buttons -->
			<div class="flex items-center justify-center gap-2">
				<button
					onclick={restart}
					class="rounded-md bg-surface-700 px-3 py-2 text-sm font-bold text-white transition-colors hover:bg-surface-600"
				>
					⏮ Restart
				</button>
				
				<button
					onclick={prevMove}
					disabled={currentMoveIndex === 0}
					class="rounded-md bg-surface-700 px-3 py-2 text-sm font-bold text-white transition-colors hover:bg-surface-600 disabled:opacity-50"
				>
					⏪ Prev
				</button>

				{#if isPlaying}
					<button
						onclick={pause}
						class="rounded-md bg-orange-500 px-4 py-2 font-bold text-white transition-colors hover:bg-orange-600"
					>
						⏸ Pause
					</button>
				{:else}
					<button
						onclick={play}
						class="rounded-md bg-emerald-500 px-4 py-2 font-bold text-white transition-colors hover:bg-emerald-600"
					>
						▶ Play
					</button>
				{/if}

				<button
					onclick={nextMove}
					disabled={currentMoveIndex >= totalMoves}
					class="rounded-md bg-surface-700 px-3 py-2 text-sm font-bold text-white transition-colors hover:bg-surface-600 disabled:opacity-50"
				>
					Next ⏩
				</button>
			</div>

			<!-- Speed Controls -->
			<div class="flex items-center justify-center gap-2">
				<span class="text-sm text-surface-400">Speed:</span>
				{#each [0.5, 1, 2, 4] as speed}
					<button
						onclick={() => setSpeed(speed)}
						class="rounded-md px-3 py-1 text-sm font-bold transition-colors
							{playbackSpeed === speed
							? 'bg-purple-500 text-white'
							: 'bg-surface-700 text-surface-300 hover:bg-surface-600'}"
					>
						{speed}x
					</button>
				{/each}
			</div>
		</div>
	{/if}
</div>

<style>
	.replay-container {
		margin: 0 auto;
		text-align: center;
		overflow: visible;
		transition: all 0.2s ease-in-out;
	}

	.replay-container.lg {
		max-width: 555px;
	}

	.replay-container.md {
		max-width: 460px;
	}

	.replay-container.sm {
		max-width: 370px;
	}

	input[type='range'] {
		-webkit-appearance: none;
		appearance: none;
		background: #3a3a3c;
		height: 8px;
		border-radius: 4px;
		outline: none;
	}

	input[type='range']::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 20px;
		height: 20px;
		background: #8b5cf6;
		cursor: pointer;
		border-radius: 50%;
	}

	input[type='range']::-moz-range-thumb {
		width: 20px;
		height: 20px;
		background: #8b5cf6;
		cursor: pointer;
		border-radius: 50%;
		border: none;
	}
</style>
