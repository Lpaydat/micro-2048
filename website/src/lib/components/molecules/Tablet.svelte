<script lang="ts">
  import type { Tablet } from "$lib/game/models";
  import Tile from "../atoms/Tile.svelte";

  export let tablet: Tablet;
  export let size: 'sm' | 'md' | 'lg' = 'lg';

  const sizeConfig = {
    sm: { tile: 80, gap: 10, wrapper: 10 },
    md: { tile: 100, gap: 12, wrapper: 12 },
    lg: { tile: 120, gap: 15, wrapper: 15 }
  };

  $: currentSize = sizeConfig[size];
</script>

<div 
  class="grid wrapper"
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