<script lang="ts">
	import '../app.css';
	import { getClient } from '$lib/client';
	import { onMount, type Snippet } from 'svelte';
	import { initializeStores, Modal, Drawer, getDrawerStore } from '@skeletonlabs/skeleton';
	import { computePosition, autoUpdate, offset, shift, flip, arrow } from '@floating-ui/dom';
	import { storePopup } from '@skeletonlabs/skeleton';
	import type { ModalComponent } from '@skeletonlabs/skeleton';

	import { setContextClient } from '@urql/svelte';
	import { userStore } from '$lib/stores/userStore';
	import EliminationLeaderboard from '$lib/components/organisms/EliminationLeaderboard.svelte';
	import HowToPlayElimination from '$lib/components/organisms/HowToPlayElimination.svelte';
	import HowToPlay2048 from '$lib/components/organisms/HowToPlay2048.svelte';
	import { chainId } from '$lib/constants';
	import SideLeaderboard from '$lib/components/organisms/SideLeaderboard.svelte';
	import { getPlayerInfo } from '$lib/graphql/queries/getPlayerInfo';
	import { logout } from '$lib/utils/logout';

	interface Props {
		children: Snippet;
	}

	let { children }: Props = $props();

	initializeStores();

	const client = getClient(chainId);
	setContextClient(client);

	let playerUsername: string | null = $state(null);
	let playerPasswordHash: string | null = $state(null);
	let playerChainId: string | null = $state(null);
	let playerIsMod: string | null = $state(null);

	onMount(() => {
		playerUsername = localStorage.getItem('username');
		playerPasswordHash = localStorage.getItem('passwordHash');
		playerChainId = localStorage.getItem('chainId');
		playerIsMod = localStorage.getItem('isMod');
	});

	const player = $derived(playerUsername ? getPlayerInfo(client, playerUsername) : null);

	let isPlayerInfoLoaded = $state(false);
	$effect(() => {
		if (typeof window !== 'undefined' && $player?.data?.player && !isPlayerInfoLoaded) {
			isPlayerInfoLoaded = true;

			playerUsername = localStorage.getItem('username');
			playerPasswordHash = localStorage.getItem('passwordHash');
			playerChainId = localStorage.getItem('chainId');
			playerIsMod = localStorage.getItem('isMod');

			userStore.update((store) => ({
				...store,
				username: playerUsername,
				passwordHash: playerPasswordHash,
				chainId: playerChainId,
				isMod: playerIsMod === 'true'
			}));
		} else if (!$player?.data && !$player?.fetching && playerUsername) {
			logout();
			isPlayerInfoLoaded = false;
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
		<SideLeaderboard
			isFullScreen
			rankers={$drawerStore.meta?.rankers}
			currentScore={$drawerStore.meta?.currentScore}
			leaderboardId={$drawerStore.meta?.leaderboardId}
		/>
	{/if}
</Drawer>

{@render children?.()}
