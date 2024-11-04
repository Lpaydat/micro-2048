<script lang="ts">
	import HeroImage from "../molecules/HeroImage.svelte";
	import GameModeSelector from "../organisms/GameModeSelector.svelte";
	import HeaderNav from "../organisms/HeaderNav.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserSidebar from "../organisms/UserSidebar.svelte";

    export let username: string | undefined;
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <UserSidebar bind:username />
    </svelte:fragment>

    <svelte:fragment slot="main">
        {#if username}
            <div class="p-4 flex items-center justify-center">
                <GameModeSelector />
            </div>
        {:else}
            <HeaderNav />
            <div class="scale-[240%] rotate-12 ml-[0vw] mt-[55vh]">
                <HeroImage />
            </div>
        {/if}
    </svelte:fragment>
</MainTemplate>
