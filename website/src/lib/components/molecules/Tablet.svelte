<script lang="ts">
	import type { Tablet } from '$lib/game/models';
	import { generateTabletFromMatrix } from '$lib/game/utils';
	import Tile from '../atoms/Tile.svelte';

	interface Props {
		tablet?: Tablet;
		size?: 'sm' | 'md' | 'lg';
	}

	const defaultTablet = generateTabletFromMatrix([
		[0, 0, 0, 0],
		[0, 0, 0, 0],
		[0, 0, 0, 0],
		[0, 0, 0, 0]
	]);

	let { tablet = defaultTablet, size = 'lg' }: Props = $props();

	const sizeConfig = {
		sm: { tile: 80, gap: 10, wrapper: 10 },
		md: { tile: 100, gap: 12.5, wrapper: 12.5 },
		lg: { tile: 120, gap: 15, wrapper: 15 }
	};

	const currentSize = $derived(sizeConfig[size]);
</script>

<div
	class="wrapper grid"
	style="
    --tile-size: {currentSize.tile}px; 
    --gap-size: {currentSize.gap}px;
    --wrapper-padding: {currentSize.wrapper}px;
  "
>
	{#each [...Array(tablet.length ** 2).keys()] as box}
		<div class="box box-{box}"></div>
	{/each}
	<div class="tiles">
		{#each tablet.flatMap((row) => row) as tile}
			{#if tile.position}
				<Tile {tile} {size} />
			{/if}
		{/each}
	</div>
</div>

<style>
	.grid {
		display: grid;
		grid-gap: var(--gap-size);
		grid-template-columns: repeat(4, var(--tile-size));
		grid-template-rows: repeat(4, var(--tile-size));
		grid-auto-flow: row;
	}
	.wrapper {
		position: relative;
		padding: var(--wrapper-padding);
		background-color: #bbada0;
		border-radius: 6px;
		width: fit-content;
		margin: 0 auto;
	}

	.tiles {
		position: absolute;
		height: 100%;
		width: 100%;
	}

	.box {
		background-color: #cdc1b4;
	}
</style>
