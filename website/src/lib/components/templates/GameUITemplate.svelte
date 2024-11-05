<script lang="ts">
    import { setContextClient } from '@urql/svelte';
    import { getClient } from '$lib/client';

	import Game from "../organisms/Game.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import Leaderboard from '../organisms/Leaderboard.svelte';
	import Brand from '../molecules/Brand.svelte';

    const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
    const applicationId = '3326df61687858af29d28b71270a6ba8c259d7fb3a666ddef25f30038648d78cf933246bda05dd51d20395557cc6efa1b7fad0d5d0027d371059fc33978674bbe476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65c10000000000000000000000';
    const port = '8080';

    const client = getClient(chainId, applicationId, port);
    setContextClient(client);
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <Brand />
        <Leaderboard />
    </svelte:fragment>

    <svelte:fragment slot="main">
        <div class="flex justify-center items-center h-full">
            <Game />
        </div>
    </svelte:fragment>
</MainTemplate>
