<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		title: Snippet;
		description: Snippet;
	}

	let { title, description }: Props = $props();
</script>

<div
	role="button"
	class="card variant-filled-surface flex h-full w-full cursor-pointer flex-col rounded-lg p-4 transition-all duration-200 hover:scale-105 hover:shadow-xl"
>
	<header class="card-header pb-2 md:pb-4 text-center text-lg md:text-2xl font-bold">
		{@render title()}
	</header>

	<section class="flex-grow p-4 text-sm md:text-base">
		{@render description()}
	</section>
</div>
