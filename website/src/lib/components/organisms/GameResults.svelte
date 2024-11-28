<script lang="ts">
	import ListItem from '../molecules/LeaderboardItem.svelte';
	import type { PlayerStats } from '$lib/types/leaderboard';

	interface Props {
		gameLeaderboard: PlayerStats[];
		player: string;
	}

	let { gameLeaderboard = [], player }: Props = $props();

	const sortedGameLeaderboard = $derived(
		gameLeaderboard
			?.slice() // Create a shallow copy to avoid mutating the original array
			.sort((a, b) => b.score - a.score) // Sort by score in descending order
			.map((player, index) => ({ ...player, rank: index + 1 })) // Add rank based on sorted position
	);
</script>

<div class="w-full max-w-md rounded-lg bg-[#FAF8EF] p-6 shadow-lg">
	<h2 class="mb-4 text-center text-2xl font-bold">Game Over</h2>
	<ul class="list-none p-0">
		{#each sortedGameLeaderboard as { rank, username, score }}
			<ListItem boardUrl="" {rank} name={username} isCurrentPlayer={username === player} {score} />
		{/each}
	</ul>
</div>
