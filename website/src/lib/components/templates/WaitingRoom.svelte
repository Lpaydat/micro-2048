<script lang="ts">
    import Clock from 'lucide-svelte/icons/clock-4';

	import ActionButton from '../atoms/ActionButton.svelte';
	import PageHeader from "../molecules/PageHeader.svelte";
    import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte'
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import WaitingPlayers from "../organisms/WaitingPlayers.svelte";
	import UserSidebar from "../organisms/UserSidebar.svelte";
	import type { EliminationGameDetails } from '$lib/types/eliminationGame';

    export let data: EliminationGameDetails;

    let isHost = true;
    let username: string | undefined = 'lpaydat';
    let players: Array<string> = ['lpaydat', 'mint', 'lisa', 'moii'];
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <UserSidebar bind:username />
    </svelte:fragment>

    <svelte:fragment slot="main">
        <PageHeader title={data.name}>
            <svelte:fragment slot="icon">
                <Clock size={28} />
            </svelte:fragment>
            <svelte:fragment slot="actions">
                {#if isHost}
                    <ActionButton icon="plus" label="START GAME" on:click />
                    <ActionButton icon="plus" label="END GAME" hoverColor="danger" on:click />
                {:else}
                    <ActionButton icon="plus" label="LEAVE GAME" on:click />
                {/if}
            </svelte:fragment>
        </PageHeader>
        <GameSettingsDetails {data} />
        <WaitingPlayers {players} maxPlayers={data.maxPlayers} />
    </svelte:fragment>
</MainTemplate>
