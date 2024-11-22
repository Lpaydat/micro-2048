<script lang="ts">
	import { userStore } from "$lib/stores/userStore";
	import HeroImage from "../molecules/HeroImage.svelte";
	import GameModeSelector from "../organisms/GameModeSelector.svelte";
	import HeaderNav from "../organisms/HeaderNav.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserSidebar from "../organisms/UserSidebar.svelte";
</script>

<MainTemplate overflowHidden>
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
            <div class="relative w-full h-full">
                <div class="relative z-10">
                    <HeaderNav />
                </div>
                <div class="scale-[120%] lg:scale-[240%] rotate-12 ml-[-20vw] mt-[20vh] lg:ml-[0vw] lg:mt-[55vh]">
                    <HeroImage />
                </div>
            </div>
        {/if}
    </svelte:fragment>
</MainTemplate>
