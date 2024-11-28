<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		leftContent?: Snippet;
		rightContent?: Snippet;
	}

	let { leftContent, rightContent }: Props = $props();
</script>

<div
	class="bg-surface-50-900-token container justify-between rounded-none border-4 p-4 transition-all hover:bg-surface-100-800-token hover:border-warning-900 md:rounded-lg"
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
