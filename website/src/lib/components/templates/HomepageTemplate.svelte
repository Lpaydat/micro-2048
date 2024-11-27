<script lang="ts">
	import { userStore } from '$lib/stores/userStore';
	import HeroImage from '../molecules/HeroImage.svelte';
	import GameModeSelector from '../organisms/GameModeSelector.svelte';
	import HeaderNav from '../organisms/HeaderNav.svelte';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
</script>

<MainTemplate overflowHidden>
	<svelte:fragment slot="sidebar">
		<UserSidebar />
	</svelte:fragment>

	<svelte:fragment slot="main">
		{#if $userStore.username}
			<div class="flex h-full flex-col">
				<div class="flex flex-grow items-center justify-center">
					<GameModeSelector />
				</div>
			</div>
		{:else}
			<div class="relative h-full w-full">
				<div class="relative z-10">
					<HeaderNav />
				</div>
				<div
					class="ml-[-20vw] mt-[20vh] rotate-12 scale-[120%] lg:ml-[0vw] lg:mt-[55vh] lg:scale-[240%]"
				>
					<HeroImage />
				</div>
			</div>
		{/if}
	</svelte:fragment>
</MainTemplate>
