<script lang="ts">
	import type { GameKeys, Tablet } from '$lib/game/models';
	import { onMount, type Snippet } from 'svelte';
	import GameTablet from '../molecules/Tablet.svelte';
	import { preventDefault } from '$lib/utils/preventDefault';
	import { boardSize } from '$lib/stores/gameStore';

	export interface Props {
		tablet?: Tablet;
		canMakeMove?: boolean;
		isEnded?: boolean;
		overlayMessage?: string;
		header?: Snippet; // Snippet<[BoardSize]>;
		moveCallback?: (direction: GameKeys, timestamp: string) => void;
		boardId?: string;
		chainId?: string;
		showReplayButton?: boolean;
		onReplayClick?: () => void;
		hideOverlay?: boolean;
	}

	let {
		tablet,
		canMakeMove,
		header,
		isEnded,
		overlayMessage,
		moveCallback,
		boardId,
		chainId,
		showReplayButton = false,
		onReplayClick,
		hideOverlay = false
	}: Props = $props();

	// 🎵 Rhythm miss effect
	let showMissEffect = $state(false);
	
	// Export function to trigger miss effect from parent
	export function triggerMissEffect() {
		showMissEffect = true;
		setTimeout(() => {
			showMissEffect = false;
		}, 300);
	}

	let showOverlay = $state(true);

	// Control overlay visibility
	$effect(() => {
		if (hideOverlay) {
			// hideOverlay prop takes precedence (for inspector mode)
			showOverlay = false;
		} else if (isEnded) {
			// Show overlay for ended games (when not hidden by hideOverlay)
			showOverlay = true;
		}
	});

	const SWIPE_THRESHOLD = 50;

	let touchStartX = $state<number | null>(null);
	let touchStartY = $state<number | null>(null);
	let keyPressTime = $state<number | null>(null);
	let isRendered = $state(false);

	$effect(() => {
		if (!isRendered && tablet) {
			isRendered = true;
		}
	});

	const handleTouchStart = (event: TouchEvent) => {
		event.preventDefault();
		touchStartX = event.touches[0].clientX;
		touchStartY = event.touches[0].clientY;
	};

	const handleTouchMove = (event: TouchEvent) => {
		event.preventDefault();
	};

	const handleTouchEnd = async (event: TouchEvent) => {
		event.preventDefault();
		if (!touchStartX || !touchStartY || !canMakeMove || isEnded) return;

		const touchEndX = event.changedTouches[0].clientX;
		const touchEndY = event.changedTouches[0].clientY;

		const deltaX = touchEndX - touchStartX;
		const deltaY = touchEndY - touchStartY;

		const timestamp = Date.now().toString();

		if (Math.abs(deltaX) > Math.abs(deltaY)) {
			if (Math.abs(deltaX) >= SWIPE_THRESHOLD) {
				if (deltaX > 0) {
					moveCallback?.('ArrowRight', timestamp);
				} else {
					moveCallback?.('ArrowLeft', timestamp);
				}
			}
		} else {
			if (Math.abs(deltaY) >= SWIPE_THRESHOLD) {
				if (deltaY > 0) {
					moveCallback?.('ArrowDown', timestamp);
				} else {
					moveCallback?.('ArrowUp', timestamp);
				}
			}
		}

		touchStartX = null;
		touchStartY = null;
	};

	const handleKeydown = async (event: KeyboardEvent) => {
		if (!canMakeMove || isEnded) return;
		keyPressTime = Date.now();

		const validKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'];
		if (validKeys.includes(event.key)) {
			const timestamp = Date.now().toString();
			moveCallback?.(event.key as GameKeys, timestamp);
		}
	};

	const updateBoardSize = () => {
		const isLandscape = window.innerWidth > window.innerHeight;

		if (window.innerWidth < 375 || (isLandscape && window.innerHeight < 500)) boardSize.set('xs');
		else if (window.innerWidth < 480 || (isLandscape && window.innerHeight < 630))
			boardSize.set('sm');
		else if (window.innerWidth < 1440 || (isLandscape && window.innerHeight < 800))
			boardSize.set('md');
		else boardSize.set('lg');
	};

	onMount(() => {
		updateBoardSize();
		window.addEventListener('resize', updateBoardSize);
		return () => window.removeEventListener('resize', updateBoardSize);
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- {@render header?.(size)} -->
{@render header?.()}
{#if isRendered}
	<div
		class="board-container relative w-full {$boardSize}"
		class:miss-shake={showMissEffect}
		style="touch-action: none;"
		ontouchstart={handleTouchStart}
		ontouchmove={handleTouchMove}
		ontouchend={handleTouchEnd}
	>
		<!-- Red flash overlay for miss -->
		{#if showMissEffect}
			<div class="miss-flash-overlay"></div>
		{/if}
		<GameTablet {tablet} />
		{#if isEnded && showOverlay}
			<div class="overlay {$boardSize}">
				<div class="flex flex-col items-center gap-4">
					<p>{overlayMessage}</p>
					{#if showReplayButton && boardId && chainId}
						<button
							onclick={() => {
								showOverlay = false;
								onReplayClick?.();
							}}
							class="rounded-md bg-purple-500 px-4 py-2 text-sm font-bold text-white transition-colors hover:bg-purple-600"
						>
							🎬 Watch Replay
						</button>
					{/if}
				</div>
			</div>
		{/if}
	</div>
{:else}
	<GameTablet />
{/if}

<style>
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

	.sm.overlay {
		font-size: 1.2em;
	}

	.md.overlay {
		font-size: 1.35em;
		width: 462.5px;
	}

	.lg.overlay {
		font-size: 1.5em;
	}

	/* 🎵 Rhythm miss effect - shake animation */
	.board-container {
		position: relative;
	}

	.miss-shake {
		animation: boardShake 0.3s ease-out;
	}

	@keyframes boardShake {
		0%, 100% { transform: translateX(0); }
		10% { transform: translateX(-8px); }
		20% { transform: translateX(8px); }
		30% { transform: translateX(-6px); }
		40% { transform: translateX(6px); }
		50% { transform: translateX(-4px); }
		60% { transform: translateX(4px); }
		70% { transform: translateX(-2px); }
		80% { transform: translateX(2px); }
		90% { transform: translateX(-1px); }
	}

	/* Red flash overlay */
	.miss-flash-overlay {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-color: rgba(239, 68, 68, 0.4);
		border-radius: 6px;
		z-index: 100;
		pointer-events: none;
		animation: flashFade 0.3s ease-out forwards;
	}

	@keyframes flashFade {
		0% { opacity: 1; }
		100% { opacity: 0; }
	}
</style>
