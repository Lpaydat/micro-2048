<script lang="ts">
	import '../app.css';
	import { getClient } from '$lib/client';
	import { onMount } from 'svelte';
	import { initializeStores, Modal, Drawer, getDrawerStore } from '@skeletonlabs/skeleton';
	import { computePosition, autoUpdate, offset, shift, flip, arrow } from '@floating-ui/dom';
	import { storePopup } from '@skeletonlabs/skeleton';
	import type { ModalComponent } from '@skeletonlabs/skeleton';
			
	import { setContextClient } from '@urql/svelte';
	import { userStore } from '$lib/stores/userStore';
	import Leaderboard from '$lib/components/organisms/Leaderboard.svelte';
	import HowToPlayElimination from '$lib/components/organisms/HowToPlayElimination.svelte';
	import HowToPlay2048 from '$lib/components/organisms/HowToPlay2048.svelte';

	initializeStores();

	// const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
	// const applicationId = '6c6bd0bf320bf7cc0d2f972f1649b9e52a151f2dbfc50bbdf736242405dcb268341717e586aaace9516bd152eb494c5ac0a38db37681d643e01e90a92950513de476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';
	// const port = '8080';

	const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
	// const applicationId = '19bbd03bf228c3e8544a5efe3e1625364884f67608cfde36a795730c6e9e8fc5aab0184faf8bc428ad82f318d060d1dd1cff23499d79fb3e7a050a701814439fe476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';
	// const port = '443';
	const applicationId = 'c287dcc227680f52688fa6d2eda25eb3e2f8328eb7c54c3763716c79093cb1cf700426b48cf61c190d747bdd2d0d0b196f0e22b9165114ee0faaa10a390d949ae476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65900000000000000000000000';
	const port = '8080';

	const client = getClient(chainId, applicationId, port);
	setContextClient(client);

	onMount(() => {
		const username = localStorage.getItem('username');
		const passwordHash = localStorage.getItem('passwordHash');
		const chainId = localStorage.getItem('chainId');
		const highestScore = Number(localStorage.getItem('highestScore'));

		if (username && passwordHash && chainId) {
		  userStore.update(store => ({
		    ...store,
		    username,
		    passwordHash,
		    chainId,
			highestScore
		  }))
		}
	})

    const drawerStore = getDrawerStore();
	storePopup.set({ computePosition, autoUpdate, offset, shift, flip, arrow });

	const modalRegistry: Record<string, ModalComponent> = {
		'how-to-play-elimination': { ref: HowToPlayElimination },
		'how-to-play-2048': { ref: HowToPlay2048 }
	};
</script>

<Modal components={modalRegistry} />
<Drawer>
	{#if $drawerStore.id === 'mobile-user-stats'}
		<Leaderboard
			isFullScreen
			player={$drawerStore.meta?.player}
			currentRound={$drawerStore.meta?.currentRound}
			gameLeaderboard={$drawerStore.meta?.gameLeaderboard}
			roundLeaderboard={$drawerStore.meta?.roundLeaderboard}
			currentPlayerScore={$drawerStore.meta?.currentPlayerScore}
		/>
	{/if}
</Drawer>

<slot />
