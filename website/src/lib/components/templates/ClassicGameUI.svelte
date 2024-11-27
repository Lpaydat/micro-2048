<script lang="ts">
	import BlockHashes from '../molecules/BlockHashes.svelte';
	import Game from '../organisms/Game.svelte';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import { userStore } from '$lib/stores/userStore';
	import { isHashesListVisible } from '$lib/stores/hashesStore';

	let canMakeMove = !!$userStore.username;
	$: chainId = $userStore.chainId;
</script>

<MainTemplate mainCenter>
	<svelte:fragment slot="sidebar">
		<UserSidebar />
	</svelte:fragment>

	<svelte:fragment slot="main">
		<div class="flex h-full items-center justify-center">
			<div class="my-auto w-full max-w-2xl lg:pb-28">
				<Game
					player={$userStore.username ?? ''}
					playerChainId={chainId as string}
					canStartNewGame={!!$userStore.username}
					showBestScore
					{canMakeMove}
				/>
			</div>
		</div>
		{#if $isHashesListVisible}
			<BlockHashes />
		{/if}
	</svelte:fragment>
</MainTemplate>
