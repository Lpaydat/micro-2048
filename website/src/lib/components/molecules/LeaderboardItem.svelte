<script lang="ts">
	import { page } from '$app/stores';
	import { getDrawerStore } from '@skeletonlabs/skeleton';

	const drawerStore = getDrawerStore();

	interface Props {
		rank: number;
		name: string;
		score: number;
		isEliminated?: boolean;
		isCurrentPlayer?: boolean;
		boardId: string;
		boardUrl: string;
	}

	let { rank, name, score, isEliminated, isCurrentPlayer, boardId, boardUrl }: Props = $props();

	const paramsBoardId = $derived($page.url.searchParams.get('boardId') ?? $page.params.boardId ??  '');
	const isActiveBoard = $derived(paramsBoardId === boardId);
	const color = $derived(isEliminated ? 'bg-[#F3F3F3]' : 'bg-[#EEE4DA]');
	const currentPlayerStyle = $derived(
		isCurrentPlayer
			? isEliminated
				? '!border-l-[#E57373] font-bold bg-[#FFCDD2]'
				: '!border-l-blue-500 font-bold bg-[#FFD700]'
			: ''
	);
	const selectedPlayerStyle = $derived(isActiveBoard ? '!border-l-orange-500 bg-teal-500/20' : '');
	const specStyle = $derived(currentPlayerStyle || selectedPlayerStyle);
	const commonClasses = $derived(
		`flex justify-between snap-start items-center p-3 pl-2 w-full ${color} rounded-sm shadow-md relative border-l-4 border-transparent ${specStyle}`
	);

	const displayRank = $derived(rank === 1 ? `👑` : rank);
</script>

{#if boardUrl}
	<a href={boardUrl} class="{commonClasses}" onclick={() => drawerStore.close()}>
		<span class="w-8 text-left font-bold text-surface-700">{displayRank}</span>
		<span class="ml-4 flex-1 truncate text-left text-surface-800">{name}</span>
		<span class="w-20 text-right font-mono text-surface-600">{score}</span>
	</a>
{:else}
	<div class="{commonClasses}">
		<span class="w-8 text-left font-bold text-surface-700">{displayRank}</span>
		<span class="ml-4 flex-1 truncate text-left text-surface-800">{name}</span>
		<span class="w-20 text-right font-mono text-surface-600">{score}</span>
	</div>
{/if}

<style>
	a,
	div {
		border-bottom: 1px solid #bbada0;
		transition:
			background-color 0.3s ease,
			transform 0.3s ease;
		position: relative;
		cursor: pointer;
		scroll-snap-align: start;
	}
	a:hover,
	div:hover {
		background-color: #d3c4b1;
	}
	span {
		padding: 0 8px;
	}
</style>
