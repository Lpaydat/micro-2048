<script lang="ts">
	export let rank: number;
	export let name: string;
	export let score: number;
	export let isEliminated: boolean = false;
	export let isCurrentPlayer: boolean = false;
	export let isSelectedPlayer: boolean = false;
	export let boardUrl: string;

	$: color = isEliminated ? 'bg-[#F3F3F3]' : 'bg-[#EEE4DA]';
	$: currentPlayerStyle = isCurrentPlayer
		? isEliminated
			? '!border-l-[#E57373] font-bold bg-[#FFCDD2]'
			: '!border-l-blue-500 font-bold bg-[#FFD700]'
		: '';
	$: selectedPlayerStyle = isSelectedPlayer ? '!border-l-orange-500' : '';
	$: commonClasses = `flex justify-between snap-start items-center p-3 pl-2 w-full ${color} rounded-sm shadow-md relative border-l-4 border-transparent ${currentPlayerStyle} ${selectedPlayerStyle}`;
</script>

{#if boardUrl}
	<a href={boardUrl} class={commonClasses}>
		<span class="w-12 text-left font-bold text-surface-700">#{rank}</span>
		<span class="ml-4 flex-1 text-left text-surface-800">{name}</span>
		<span class="w-16 text-right text-surface-600">{score}</span>
	</a>
{:else}
	<div class={commonClasses}>
		<span class="w-12 text-left font-bold text-surface-700">#{rank}</span>
		<span class="ml-4 flex-1 text-left text-surface-800">{name}</span>
		<span class="w-16 text-right text-surface-600">{score}</span>
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
	}
	a:hover,
	div:hover {
		background-color: #d3c4b1;
	}
	span {
		padding: 0 8px;
	}
</style>
