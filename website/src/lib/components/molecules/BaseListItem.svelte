<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		leftContent?: Snippet;
		rightContent?: Snippet;
		className?: string;
	}

	let { leftContent, rightContent, className }: Props = $props();
</script>

<div
	class="bg-surface-50-900-token hover:bg-surface-100-800-token hover:border-warning-900 container mx-auto w-full max-w-4xl justify-between rounded-none border-4 p-4 transition-all md:rounded-lg {className}"
>
	<div class="flex-1">
		{@render leftContent?.()}
	</div>
	<div class="ms-auto">
		{@render rightContent?.()}
	</div>
</div>

<style>
	.container {
		display: flex;
		flex-direction: row;
	}

	@media (max-width: 768px) {
		.container {
			flex-direction: column;
		}
	}
</style>
