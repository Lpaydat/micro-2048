<script lang="ts">
	import { getDrawerStore } from '@skeletonlabs/skeleton';
	import type { DrawerSettings } from '@skeletonlabs/skeleton';
	import HeaderRankButton from '../molecules/HeaderRankButton.svelte';

	interface Props {
		player: string;
		rankers: { username: string; score: number; rank: number; boardId: string }[];
	}

	let { player, rankers = [] }: Props = $props();

	const currentPlayerIndex = $derived(rankers?.findIndex((p) => p.username === player));
	const currentPlayer = $derived(
		currentPlayerIndex !== -1 ? rankers[currentPlayerIndex] : undefined
	);
	const userRank = $derived(currentPlayerIndex + 1);
	const userScore = $derived(currentPlayer?.score ?? 0);

	const drawerStore = getDrawerStore();
	const drawerSettings = $derived<DrawerSettings>({
		id: 'mobile-ranker-stats',
		position: 'bottom',
		bgDrawer: 'bg-[#FAF8EF]',
		bgBackdrop: 'bg-black/60',
		height: 'h-[80vh]',
		meta: {
			player,
			rankers
		}
	});
</script>

<HeaderRankButton {userRank} {userScore} onclick={() => drawerStore.open(drawerSettings)} />
