<script lang="ts">
	interface Props {
		username: string;
		fontSize?: string;
	}

	let { username, fontSize = 'text-base' }: Props = $props();
</script>

<div
	class="flex items-center justify-center truncate rounded-lg bg-black/50 px-3 py-2 font-bold text-[#f67c5f] {fontSize}"
	style="font-family: 'Clear Sans', 'Arial', sans-serif; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;"
>
	{username ?? 'Please Login'}
</div>
