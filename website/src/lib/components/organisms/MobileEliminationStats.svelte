<script lang="ts">
	import type { PlayerStats, RoundResults } from '$lib/types/leaderboard';
	import { getDrawerStore } from '@skeletonlabs/skeleton';
	import type { DrawerSettings } from '@skeletonlabs/skeleton';
	import HeaderRankButton from '../molecules/HeaderRankButton.svelte';

	interface Props {
		currentRound?: number;
		player: string;
		gameLeaderboard?: PlayerStats[];
		roundLeaderboard?: RoundResults;
		currentPlayerScore?: number;
	}

	let {
		currentRound = 1,
		player,
		gameLeaderboard = [],
		roundLeaderboard,
		currentPlayerScore = 0
	}: Props = $props();

	const rlb = $derived(
		roundLeaderboard ?? {
			round: 0,
			players: [],
			eliminatedPlayers: []
		}
	);

	const combinedRoundLeaderboard = $derived(
		[...rlb.players, ...rlb.eliminatedPlayers]
			.map((p) => ({
				...p,
				isEliminated: rlb.eliminatedPlayers.includes(p),
				score: p.username === player ? currentPlayerScore : p.score
			}))
			.sort((a, b) => b.score - a.score)
			.map((player, index) => ({ ...player, rank: index + 1 }))
	);

	const currentPlayer = $derived(combinedRoundLeaderboard?.find((p) => p.username === player));
	const userRank = $derived(currentPlayer?.rank ?? 0);
	const userScore = $derived(currentPlayer?.score ?? 0);

	const drawerStore = getDrawerStore();
	const drawerSettings = $derived<DrawerSettings>({
		id: 'mobile-elimination-stats',
		position: 'bottom',
		bgDrawer: 'bg-[#FAF8EF]',
		bgBackdrop: 'bg-black/60',
		height: 'h-[80vh]',
		meta: {
			player,
			gameLeaderboard,
			currentRound,
			currentPlayerScore,
			roundLeaderboard
		}
	});
</script>

<HeaderRankButton {userRank} {userScore} onclick={() => drawerStore.open(drawerSettings)} />
