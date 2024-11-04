<script lang="ts">
	import Brand from "../molecules/Brand.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
    import GameSettingsDetails from "../organisms/GameSettingsDetails.svelte"
	import WaitingPlayers from "../organisms/WaitingPlayers.svelte";

    let isHost = false;
    let gameName = "";
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <Brand />
        <GameSettingsDetails />
        {#if isHost}
        <button class="btn btn-primary">Start Game</button>
        {/if}
    </svelte:fragment>

    <svelte:fragment slot="main">
    <div>
        <div>{gameName}</div>
        <button class="btn btn-primary">Leave</button>
        </div>
        <WaitingPlayers />
    </svelte:fragment>
</MainTemplate>
