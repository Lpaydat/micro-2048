<script lang="ts">
    import Brand from "../molecules/Brand.svelte";
	import Game from "../organisms/Game.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
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
