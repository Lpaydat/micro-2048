<script lang="ts">
    import Clock from 'lucide-svelte/icons/clock-4';
	import PageHeader from "../molecules/PageHeader.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
    import GameSettingsDetails from "../organisms/GameSettingsDetails.svelte"
	import WaitingPlayers from "../organisms/WaitingPlayers.svelte";
	import UserSidebar from "../organisms/UserSidebar.svelte";

    let isHost = false;
    let gameName = "2048 Battle Royale";
    let username: string | undefined = 'lpaydat';
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <UserSidebar bind:username />
        {#if isHost}
            <button class="btn btn-primary">Start Game</button>
        {/if}
    </svelte:fragment>

    <svelte:fragment slot="main">
        <PageHeader title={gameName}>
            <svelte:fragment slot="icon">
                <Clock size={28} />
            </svelte:fragment>
        </PageHeader>
        <div>
            <div>{gameName}</div>
            <GameSettingsDetails />
            <button class="btn btn-primary">Leave</button>
        </div>
        <WaitingPlayers />
    </svelte:fragment>
</MainTemplate>
