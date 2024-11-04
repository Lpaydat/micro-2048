<script lang="ts">
	import Brand from "../organisms/Brand.svelte";
	import GameModeSelector from "../organisms/GameModeSelector.svelte";
	import HeaderNav from "../organisms/HeaderNav.svelte";
	import HeroImage from "../organisms/HeroImage.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserLogin from "../organisms/UserLogin.svelte";

    let isLoggedIn = false;
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <Brand />
        <div class="flex items-center justify-center mt-24">
            <UserLogin username={isLoggedIn ? 'hello' : undefined} />
        </div>
    </svelte:fragment>

    <svelte:fragment slot="main">
        {#if isLoggedIn}
            <GameModeSelector />
        {:else}
            <HeaderNav />
            <div class="scale-[240%] rotate-12 ml-[0vw] mt-[55vh]">
                <HeroImage />
            </div>
        {/if}
    </svelte:fragment>
</MainTemplate>
