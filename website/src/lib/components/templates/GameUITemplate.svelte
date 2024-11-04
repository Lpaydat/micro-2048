<script lang="ts">
	import Game from "../organisms/Game.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
    import Brand from "../organisms/Brand.svelte";
    import Leaderboard from "../organisms/Leaderboard.svelte";

    let isSinglePlayer = true;
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        {#if isSinglePlayer}
        <Brand />
    {:else}
        <Leaderboard />
        {/if}
    </svelte:fragment>

    <svelte:fragment slot="main">
        <Game />
    </svelte:fragment>
</MainTemplate>
