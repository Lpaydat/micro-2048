<script lang="ts">
	import { page } from '$app/stores';

	interface Props {
		rank: number;
		player: { username: string; score: number; boardId: string };
	}

	let { rank, player }: Props = $props();

	const leaderboardId = $derived($page.params.leaderboardId);
	const extraParams = $derived(leaderboardId ? `&leaderboardId=${leaderboardId}` : '');
</script>

<a
	href={`/game?boardId=${player.boardId}${extraParams}`}
	class="ranker-card flex snap-start items-center justify-between rounded-lg border-4 border-white bg-white p-3 text-sm shadow-lg transition-colors duration-100 hover:border-cyan-600 hover:bg-gray-100 lg:text-lg"
>
	<span class="w-1/12 text-left font-bold text-gray-800">{rank}</span>
	<span class="w-5/12 overflow-hidden text-ellipsis whitespace-nowrap font-bold text-orange-600"
		>{player.username}</span
	>
	<span class="w-3/12 font-mono text-gray-500">{player.boardId}</span>
	<span class="w-3/12 text-right font-mono font-bold text-green-500">{player.score}</span>
</a>
