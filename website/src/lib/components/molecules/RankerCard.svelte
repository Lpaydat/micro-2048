<script lang="ts">
    export let rank: number;
    export let player: { username: string; score: number; boardId: string };
</script>

<a href={`/game?boardId=${player.boardId}`} class="ranker-card bg-white text-sm lg:text-lg p-3 border-4 border-white hover:border-cyan-600 rounded-lg shadow-lg flex justify-between items-center transition-colors duration-100 hover:bg-gray-100">
    <span class="w-1/12 text-left font-bold text-gray-800">{rank}</span>
    <span class="w-5/12 font-bold text-orange-600">{player.username}</span>
    <span class="w-3/12 text-gray-500">{player.boardId}</span>
    <span class="w-3/12 text-right font-bold text-green-500">{player.score}</span>
</a>