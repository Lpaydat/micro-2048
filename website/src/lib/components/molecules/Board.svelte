<script lang="ts">
  import BoardTile from "../atoms/BoardTile.svelte";

  export let board: number[][];
</script>

<style>
  .grid {
    display: grid;
    grid-gap: 15px;
    grid-template-columns: repeat(4, 120px);
    grid-template-rows: repeat(4, 120px);
    grid-auto-flow: row;
  }
  .wrapper {
    position: relative;
    padding: 15px;
    background-color: #bbada0;
    border-radius: 6px;
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

<div class="grid wrapper">
  {#if board}
    {#each [...Array(board.length ** 2).keys()] as box}
      <div class="box box-{box}"></div>
    {/each}
    <div class="tiles">
      {#each board.reduce((acc, row) => acc.concat(row), []) as value, index}
        <BoardTile {value} {index} />
      {/each}
    </div>
  {/if}
</div>
