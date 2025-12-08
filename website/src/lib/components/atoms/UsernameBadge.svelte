<script lang="ts">
	interface Props {
		username: string;
		fontSize?: string;
	}

	let { username, fontSize = 'text-base' }: Props = $props();
</script>

<div
	class="flex min-w-0 max-w-full items-center justify-center rounded-lg bg-black/50 px-3 py-2 font-bold text-[#f67c5f] {fontSize}"
	style="font-family: 'Clear Sans', 'Arial', sans-serif;"
>
	<span class="truncate">
		{username ?? 'Please Login'}
	</span>
</div>
