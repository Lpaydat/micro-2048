<script lang="ts">
	import MainTemplate from "../organisms/MainTemplate.svelte";
    import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte'
	import WaitingPlayers from "../organisms/WaitingPlayers.svelte";
	import Brand from "../organisms/Brand.svelte";

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
