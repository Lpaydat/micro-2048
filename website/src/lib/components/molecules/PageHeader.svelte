<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		icon?: Snippet;
		actions?: Snippet;
		title: string;
		prevPage?: string;
		color?:
			| 'green'
			| 'orange'
			| 'red'
			| 'blue'
			| 'purple'
			| 'yellow'
			| 'pink'
			| 'gray'
			| 'teal'
			| 'cyan';
	}

	let { icon, actions, title, prevPage, color = 'green' }: Props = $props();

	const headerRound = $derived(!prevPage ? 'md:rounded-bl-lg mb-4 md:mb-0' : '');

	const colorClass = {
		green: 'bg-green-600',
		orange: 'bg-orange-600',
		red: 'bg-red-600',
		blue: 'bg-blue-600',
		purple: 'bg-purple-600',
		yellow: 'bg-yellow-600',
		pink: 'bg-pink-600',
		gray: 'bg-gray-600',
		teal: 'bg-teal-600',
		cyan: 'bg-cyan-600'
	};
</script>

<div class="flex w-full flex-col">
	<div
		class="flex w-full items-center bg-gradient-to-r from-warning-500 to-warning-600 font-bold text-white {headerRound} relative p-2 shadow-md transition-all md:p-4"
	>
		<div
			class="header flex items-center justify-center gap-2 rounded-lg px-2 py-2 text-sm font-bold transition-all lg:gap-4 lg:px-4 lg:text-sm {colorClass[
				color
			]}"
		>
			{@render icon?.()}
			<span class="text-md tracking-wider lg:text-2xl">{title}</span>
		</div>

		<div class="ms-auto flex items-center gap-2">
			{@render actions?.()}
		</div>
	</div>

	{#if prevPage}
		<div class="flex">
			<a
				href={prevPage}
				class="inline-block rounded-br-md bg-surface-400 px-3 py-2 text-xs text-white transition-colors hover:bg-surface-800 lg:rounded-b-md lg:text-sm"
			>
				Back
			</a>
		</div>
	{/if}
</div>

<style>
	.header {
		border: 2px solid #fff;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
	}
</style>
