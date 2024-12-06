<script lang="ts">
	import type { TileContent } from '$lib/game/models';
	import { tweened, spring } from 'svelte/motion';
	import { cubicOut } from 'svelte/easing';
	import { onMount } from 'svelte';
	import { boardSize } from '$lib/stores/gameStore';

	interface Props {
		tile: TileContent;
	}

	let { tile }: Props = $props();

	const sizeConfig = {
		xs: { tile: 216.5, gap: 7.5, fontSize: { default: 33.75, medium: 22.5, small: 15 } },
		sm: { tile: 270, gap: 10, fontSize: { default: 45, medium: 30, small: 20 } },
		md: { tile: 337.5, gap: 12, fontSize: { default: 55, medium: 37.5, small: 27.5 } },
		lg: { tile: 405, gap: 15, fontSize: { default: 65, medium: 40, small: 30 } }
	};

	const posTop = tile.position?.top ?? 0;
	const prevPosTop = tile.prevPosition?.top ?? 0;
	const posLeft = tile.position?.left ?? 0;
	const prevPosLeft = tile.prevPosition?.left ?? 0;

	const topTweened = tweened((tile.new ? posTop : prevPosTop) / 3, {
		duration: 75,
		easing: cubicOut
	});

	const leftTweened = tweened((tile.new ? posLeft : prevPosLeft) / 3, {
		duration: 75,
		easing: cubicOut
	});

	const mergeSpring = spring(
		{ scale: 0 },
		{
			stiffness: 0.5,
			damping: 0.8
		}
	);

	let mergedValue = $state(0);

	// Based on composite rule of three
	const currentSize = $derived(sizeConfig[$boardSize]);
	const top = $derived($topTweened * currentSize.tile + currentSize.gap);
	const left = $derived($leftTweened * currentSize.tile + currentSize.gap);
	const wasMerged = $derived(tile.merged && mergedValue !== tile.value);

	$effect(() => {
		if (wasMerged) {
			mergeSpring.set({ scale: 1 });
		}

		if ($mergeSpring.scale === 1) {
			mergedValue = tile.value;
			mergeSpring.set({ scale: 0 });
		}
	});

	onMount(() => {
		topTweened.set(posTop / 3);
		leftTweened.set(posLeft / 3);
	});

	const scale = (node: any, { duration }: any) => {
		return {
			duration,
			css: (t: any) => {
				const eased = tile.new ? cubicOut(t) : 1;

				return `
          transform: scale(${eased});
          `;
			}
		};
	};

	// Add this function to determine font size
	const getFontSize = (value: number, size: 'xs' | 'sm' | 'md' | 'lg'): number => {
		const config = sizeConfig[size].fontSize;
		if (value <= 6) return config.default;
		if (value <= 9) return config.medium;
		return config.small;
	};
</script>

<div
	class="tile tile-{tile.value} size-{$boardSize}"
	style="top: {top}px; left: {left}px; font-size: {getFontSize(
		tile.value,
		$boardSize
	)}px; transform: scale({wasMerged ? $mergeSpring.scale : 1})"
	in:scale={{ duration: 100 }}
>
	{tile.value !== 0 ? 2 ** tile.value : ''}
</div>

<style>
	.size-xs {
		line-height: 64px;
		width: 64px;
		height: 64px;
	}
	.size-sm {
		line-height: 80px;
		width: 80px;
		height: 80px;
	}
	.size-md {
		line-height: 100px;
		width: 100px;
		height: 100px;
	}
	.size-lg {
		line-height: 120px;
		width: 120px;
		height: 120px;
	}
	.tile {
		text-align: center;
		position: absolute;
		border-radius: 6px;
		font-weight: bold;
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
		transition: 50ms ease-in-out;
		user-select: none;
	}
	.tile-1 {
		color: #776e65;
		background: #eee4da;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0),
			inset 0 0 0 1px rgba(255, 255, 255, 0);
	}
	.tile-2 {
		color: #776e65;
		background: #ede0c8;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0),
			inset 0 0 0 1px rgba(255, 255, 255, 0);
	}
	.tile-3 {
		color: #f9f6f2;
		background: #f2b179;
	}
	.tile-4 {
		color: #f9f6f2;
		background: #f59563;
	}
	.tile-5 {
		color: #f9f6f2;
		background: #f67c5f;
	}
	.tile-6 {
		color: #f9f6f2;
		background: #f65e3b;
	}
	.tile-7 {
		color: #f9f6f2;
		background: #edcf72;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.2381),
			inset 0 0 0 1px rgba(255, 255, 255, 0.14286);
	}
	.tile-8 {
		color: #f9f6f2;
		background: #edcc61;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.31746),
			inset 0 0 0 1px rgba(255, 255, 255, 0.19048);
	}
	.tile-9 {
		color: #f9f6f2;
		background: #edc850;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.39683),
			inset 0 0 0 1px rgba(255, 255, 255, 0.2381);
	}
	.tile-10 {
		color: #f9f6f2;
		background: #edc53f;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.47619),
			inset 0 0 0 1px rgba(255, 255, 255, 0.28571);
	}
	.tile-11 {
		color: #f9f6f2;
		background: #f46573;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.55556),
			inset 0 0 0 1px rgba(255, 255, 255, 0.33333);
	}
	.tile-12 {
		color: #f9f6f2;
		background: #f14b61;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.55556),
			inset 0 0 0 1px rgba(255, 255, 255, 0.33333);
	}
	.tile-13 {
		color: #f9f6f2;
		background: #e9443d;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.55556),
			inset 0 0 0 1px rgba(255, 255, 255, 0.33333);
	}
	.tile-14 {
		color: #f9f6f2;
		background: #72b3db;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.55556),
			inset 0 0 0 1px rgba(255, 255, 255, 0.33333);
	}
	.tile-15 {
		color: #f9f6f2;
		background: #5da0e4;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.55556),
			inset 0 0 0 1px rgba(255, 255, 255, 0.33333);
	}
	.tile-16 {
		color: #f9f6f2;
		background: #027dc0;
		box-shadow:
			0 0 30px 10px rgba(243, 215, 116, 0.55556),
			inset 0 0 0 1px rgba(255, 255, 255, 0.33333);
	}
</style>
