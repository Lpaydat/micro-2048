<script lang="ts">
  import ListItem from '../molecules/LeaderboardItem.svelte';
  import type { PlayerStats } from '$lib/types/leaderboard';

  export let gameLeaderboard: PlayerStats[] = [];
  export let player: string;

  $: sortedGameLeaderboard = gameLeaderboard
    ?.slice() // Create a shallow copy to avoid mutating the original array
    .sort((a, b) => b.score - a.score) // Sort by score in descending order
    .map((player, index) => ({ ...player, rank: index + 1 })); // Add rank based on sorted position
</script>

<div class="bg-[#FAF8EF] p-6 rounded-lg shadow-lg max-w-md w-full">
    <h2 class="text-2xl font-bold mb-4 text-center">Game Over</h2>
    <ul class="list-none p-0">
    {#each sortedGameLeaderboard as {rank, username, score}}
        <ListItem 
            boardUrl=''
            {rank}
            name={username}
            isCurrentPlayer={username === player}
            {score}
        />
    {/each}
    </ul>
</div>
