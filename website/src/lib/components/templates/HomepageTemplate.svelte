<script lang="ts">
	import { userStore } from "$lib/stores/userStore";
	import HeroImage from "../molecules/HeroImage.svelte";
	import GameModeSelector from "../organisms/GameModeSelector.svelte";
	import HeaderNav from "../organisms/HeaderNav.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserSidebar from "../organisms/UserSidebar.svelte";
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <UserSidebar />
    </svelte:fragment>

    <svelte:fragment slot="main">
        {#if $userStore.username}
            <div class="flex flex-col h-full">
                <div class="flex-grow flex items-center justify-center">
                    <GameModeSelector />
                </div>
            </div>
        {:else}
            <HeaderNav />
            <div class="scale-[240%] rotate-12 ml-[0vw] mt-[55vh]">
                <HeroImage />
            </div>
        {/if}
    </svelte:fragment>
</MainTemplate>
