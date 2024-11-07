<script lang="ts">
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
