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
	import EliminationLeaderboard from '$lib/components/organisms/EliminationLeaderboard.svelte';
	import HowToPlayElimination from '$lib/components/organisms/HowToPlayElimination.svelte';
	import HowToPlay2048 from '$lib/components/organisms/HowToPlay2048.svelte';
	import { applicationId, appVersion, chainId, port } from '$lib/constants';
	import { logout } from '$lib/utils/logout';
	import SideLeaderboard from '$lib/components/organisms/SideLeaderboard.svelte';

	initializeStores();

	const client = getClient(chainId, applicationId, port);
	setContextClient(client);

	onMount(() => {
		const version = localStorage.getItem('version');
		if (version !== appVersion) {
			// update version
			localStorage.setItem('version', appVersion);

			// force logout on new version
			logout();
		}

		const username = localStorage.getItem('username');
		const passwordHash = localStorage.getItem('passwordHash');
		const chainId = localStorage.getItem('chainId');

		if (username && passwordHash && chainId) {
			userStore.update((store) => ({
				...store,
				username,
				passwordHash,
				chainId,
			}));
		}
	});

	const drawerStore = getDrawerStore();
	storePopup.set({ computePosition, autoUpdate, offset, shift, flip, arrow });

	const modalRegistry: Record<string, ModalComponent> = {
		'how-to-play-elimination': { ref: HowToPlayElimination },
		'how-to-play-2048': { ref: HowToPlay2048 }
	};
</script>

<Modal components={modalRegistry} />
<Drawer>
	{#if $drawerStore.id === 'mobile-elimination-stats'}
		<EliminationLeaderboard
			isFullScreen
			player={$drawerStore.meta?.player}
			currentRound={$drawerStore.meta?.currentRound}
			gameLeaderboard={$drawerStore.meta?.gameLeaderboard}
			roundLeaderboard={$drawerStore.meta?.roundLeaderboard}
			currentPlayerScore={$drawerStore.meta?.currentPlayerScore}
		/>
	{/if}
	{#if $drawerStore.id === 'mobile-ranker-stats'}
		<SideLeaderboard isFullScreen rankers={$drawerStore.meta?.rankers} />
	{/if}
</Drawer>

<slot />
