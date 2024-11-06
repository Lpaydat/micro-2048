<script lang="ts">
    import { setContextClient } from '@urql/svelte';
    import { getClient } from '$lib/client';

	import Game from "../organisms/Game.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import Leaderboard from '../organisms/Leaderboard.svelte';
	import Brand from '../molecules/Brand.svelte';
	import GameSettingsDetails from '../organisms/GameSettingsDetails.svelte';
	import type { EliminationGameDetails } from '$lib/types/eliminationGame';
	import UserSidebar from '../organisms/UserSidebar.svelte';

    export let data: EliminationGameDetails;
    export let isMultiplayer: boolean = false;
    export let username: string | undefined;

    const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
    const applicationId = '8871dee65d6dcedf03f1e255fe7f8e69152ac1678170302283f357885b1f834407b28b27d287fbea47e91b785d2e24cead390e16fedc630e996fb03d21aa2219e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65a80100000000000000000000';
    const port = '8080';

    const client = getClient(chainId, applicationId, port);
    setContextClient(client);
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        {#if isMultiplayer}
            <Brand />
            <Leaderboard />
        {:else}
            <UserSidebar bind:username />
        {/if}
    </svelte:fragment>

    <svelte:fragment slot="main">
        {#if isMultiplayer}
            <GameSettingsDetails {data} numberA={data.currentRound} numberB={data.totalRounds} numberLabel="Round" />
        {/if}
        <div class="flex justify-center items-center h-full">
            <div class="w-full max-w-2xl pb-28">
                <Game />
            </div>
        </div>
    </svelte:fragment>
</MainTemplate>
