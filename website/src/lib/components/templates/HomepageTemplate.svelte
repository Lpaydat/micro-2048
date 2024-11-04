<script lang="ts">
	import Brand from "../molecules/Brand.svelte";
	import HeroImage from "../molecules/HeroImage.svelte";
	import GameModeSelector from "../organisms/GameModeSelector.svelte";
	import HeaderNav from "../organisms/HeaderNav.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserLogin from "../organisms/UserLogin.svelte";

    let isLoggedIn = true;
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <Brand />
        <div class="flex items-center justify-center mt-24 w-full">
            <UserLogin username={isLoggedIn ? 'hello' : undefined} />
        </div>
    </svelte:fragment>

    <svelte:fragment slot="main">
        {#if isLoggedIn}
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
