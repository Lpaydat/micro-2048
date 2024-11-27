<script lang="ts">
	export let rank: number;
	export let player: { username: string; score: number; boardId: string };
</script>

<a
	href={`/game?boardId=${player.boardId}`}
	class="ranker-card flex items-center justify-between rounded-lg border-4 border-white bg-white p-3 text-sm shadow-lg transition-colors duration-100 hover:border-cyan-600 hover:bg-gray-100 lg:text-lg"
>
	<span class="w-1/12 text-left font-bold text-gray-800">{rank}</span>
	<span class="w-5/12 overflow-hidden text-ellipsis whitespace-nowrap font-bold text-orange-600"
		>{player.username}</span
	>
	<span class="w-3/12 font-mono text-gray-500">{player.boardId}</span>
	<span class="w-3/12 text-right font-mono font-bold text-green-500">{player.score}</span>
</a>
