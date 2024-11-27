<script lang="ts">
	import type { PlayerStats, RoundResults } from '$lib/types/leaderboard';
	import { getDrawerStore } from '@skeletonlabs/skeleton';
	import type { DrawerSettings } from '@skeletonlabs/skeleton';

	export let currentRound: number = 1;
	export let player: string;
	export let gameLeaderboard: PlayerStats[] = [];
	export let roundLeaderboard: RoundResults | undefined;
	export let currentPlayerScore: number = 0;

	$: rlb = roundLeaderboard ?? {
		round: 0,
		players: [],
		eliminatedPlayers: []
	};

	$: combinedRoundLeaderboard = [...rlb.players, ...rlb.eliminatedPlayers]
		.map((p) => ({
			...p,
			isEliminated: rlb.eliminatedPlayers.includes(p),
			score: p.username === player ? currentPlayerScore : p.score
		}))
		.sort((a, b) => b.score - a.score)
		.map((player, index) => ({ ...player, rank: index + 1 }));

	$: currentPlayer = combinedRoundLeaderboard?.find((p) => p.username === player);
	$: userRank = currentPlayer?.rank ?? 0;
	$: userScore = currentPlayer?.score ?? 0;

	const drawerStore = getDrawerStore();
	let drawerSettings: DrawerSettings;
	$: drawerSettings = {
		id: 'mobile-user-stats',
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
	};
</script>

<button
	type="button"
	on:click={() => drawerStore.open(drawerSettings)}
	class="flex items-center justify-end rounded-lg bg-[#FAF8EF] p-2 shadow-md"
>
	<div class="flex items-center space-x-4 rounded-md bg-[#ed8d33] px-4 py-2">
		<!-- Rank -->
		<div class="flex flex-col items-center">
			<span class="text-xs uppercase text-[#EEE4DA]">Rank</span>
			<span class="font-bold text-white">#{userRank}</span>
		</div>

		<!-- Divider -->
		<div class="h-8 w-px bg-[#CDC1B4]"></div>

		<!-- Score -->
		<div class="flex flex-col items-center">
			<span class="text-xs uppercase text-[#EEE4DA]">Score</span>
			<span class="font-bold text-white">{userScore}</span>
		</div>
	</div>
</button>

<style>
	div {
		transition: all 0.3s ease;
	}
</style>
