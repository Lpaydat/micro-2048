<script lang="ts">
    export let rank: number;
    export let player: { username: string; score: number; boardId: string };
</script>

<div class="ranker-card bg-white text-sm lg:text-lg p-4 rounded-lg shadow-lg flex justify-between items-center transition-transform transform hover:scale-105">
    <span class="w-1/12 text-left font-bold text-gray-800">{rank}</span>
    <span class="w-4/12 font-bold text-orange-600">{player.username}</span>
    <span class="w-3/12 text-gray-500">{player.boardId}</span>
    <span class="w-4/12 text-right font-bold text-green-500">{player.score}</span>
</div>
