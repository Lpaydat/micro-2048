<script lang="ts">
	import { page } from '$app/stores';

	interface Props {
		rank: number;
		player: { username: string; score: number; boardId: string };
		isTournamentEnded?: boolean;
	}

	let { rank, player, isTournamentEnded = false }: Props = $props();

	const leaderboardId = $derived($page.params.leaderboardId);
	const extraParams = $derived(leaderboardId ? `&leaderboardId=${leaderboardId}` : '');
	const boardId = player.boardId.split('.')[1];

	const bgClass = $derived(isTournamentEnded ? 'bg-gray-200' : 'bg-white');
	const borderClass = $derived(isTournamentEnded ? 'border-gray-400' : 'border-white');
</script>

<a
	href={`/game?boardId=${player.boardId}${extraParams}`}
	class="ranker-card flex snap-start items-center justify-between rounded-lg border-4 {borderClass} {bgClass} p-3 text-sm shadow-lg transition-colors duration-100 hover:border-cyan-600 hover:bg-gray-100 lg:text-lg"
>
	<span class="w-1/12 text-left font-bold text-gray-800">{rank}</span>
	<span class="w-5/12 overflow-hidden text-ellipsis whitespace-nowrap font-bold text-orange-600"
		>{player.username}</span
	>
	<span class="w-3/12 font-mono text-gray-500">{boardId}</span>
	<span class="w-3/12 text-right font-mono font-bold text-green-500">{player.score}</span>
</a>
