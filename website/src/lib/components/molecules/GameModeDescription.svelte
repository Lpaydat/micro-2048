<script lang="ts">
	import DescriptionBox from '../atoms/DescriptionBox.svelte';

	interface Props {
		imageUrl: string;
		description: string;
	}

	let { imageUrl = '', description = 'Default game mode description' }: Props = $props();
</script>

<div class="mx-auto max-w-sm">
	<div class="overflow-hidden rounded-t-lg">
		<img src={imageUrl} alt="" class="w-full" />
	</div>
	<div class="flex flex-col items-center">
		<DescriptionBox {description} />
	</div>
</div>
