<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		leftContent?: Snippet;
		rightContent?: Snippet;
	}

	let { leftContent, rightContent }: Props = $props();
</script>

<div
	class="bg-surface-50-900-token flex items-center justify-between rounded-lg border-4 p-4 transition-all hover:bg-surface-100-800-token hover:border-warning-900"
>
	<div class="flex-1">
		{@render leftContent?.()}
	</div>
	{@render rightContent?.()}
</div>
