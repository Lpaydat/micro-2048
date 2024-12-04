<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import BackIcon from 'lucide-svelte/icons/chevron-left';

	interface Props {
		icon?: Snippet;
		actions?: Snippet;
		subActions?: Snippet;
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

	let { icon, actions, subActions, title, prevPage, color = 'green' }: Props = $props();

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
	const smTitle = 12;
	const iconClass = $derived(prevPage || icon ? 'ps-1 md:ps-2' : 'ps-2');

	let size = $state('md');
	const displayTitle = $derived(
		size === 'sm' && title.length > smTitle ? `${title.slice(0, smTitle)}...` : title
	);

	const updateSize = () => {
		if (window.innerWidth < 480) size = 'sm';
		else if (window.innerWidth < 1440) size = 'md';
		else size = 'lg';
	};

	onMount(() => {
		updateSize();
		window.addEventListener('resize', updateSize);
		return () => window.removeEventListener('resize', updateSize);
	});
</script>

<div class="flex w-full">
	<div
		class="flex flex-grow flex-row items-center font-bold text-white {headerRound} relative p-2 transition-all md:p-4"
	>
		<a href={prevPage ?? '#'}>
			<div
				class="header flex items-center justify-center gap-1 rounded-lg md:gap-2 {iconClass} py-2 pe-2 text-sm font-bold transition-all lg:pe-4 lg:text-sm {colorClass[
					color
				]}"
			>
				{#if prevPage}
					<BackIcon color="white" />
				{:else}
					{@render icon?.()}
				{/if}
				<span class="text-md tracking-wider lg:text-2xl" {title}>{displayTitle}</span>
			</div>
		</a>
		{@render subActions?.()}
	</div>

	<div class="me-2 flex flex-shrink-0 items-center justify-end gap-2 md:me-4">
		{@render actions?.()}
	</div>
</div>

<style>
	.header {
		border: 2px solid #fff;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
	}
</style>
