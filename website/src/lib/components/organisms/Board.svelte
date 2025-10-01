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
	}

	let { tablet, canMakeMove, header, isEnded, overlayMessage, moveCallback }: Props = $props();

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
<!-- TODO: svelte5 onevent not preventing default for touch events -->
{#if isRendered}
	<div
		class="relative w-full {$boardSize}"
		ontouchstart={preventDefault(handleTouchStart)}
		ontouchmove={preventDefault(handleTouchMove)}
		ontouchend={preventDefault(handleTouchEnd)}
	>
		<GameTablet {tablet} />
		{#if isEnded}
			<div class="overlay {$boardSize}">
				<p>{overlayMessage}</p>
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
</style>
