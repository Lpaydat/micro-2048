<script lang="ts">
	interface Props {
		opacity?: string;
	}

	let { opacity = '40' }: Props = $props();
</script>

<div
	class="fixed bottom-0 right-0 flex w-full md:w-fit justify-between rounded-md bg-black/{opacity} px-2 lg:px-4 py-1 lg:py-2 text-xs text-surface-300/80"
>
	<div class="flex-1 text-right">v1.3.0 | made with ❤️ lpaydat</div>
</div>
