<script lang="ts">
	interface Props {
		opacity?: string;
	}

	let { opacity = '40' }: Props = $props();
</script>

<div
	class="fixed bottom-0 right-0 flex w-full justify-between rounded-md md:w-fit bg-black/{opacity} px-2 py-1 text-xs text-surface-300/80 lg:px-4 lg:py-2"
>
	<div class="flex-1 text-right">
		v2.1.4 <span class="text-md text-surface-400">│</span> made with
		<span class="text-red-500">❤️</span> lpaydat
	</div>
</div>
