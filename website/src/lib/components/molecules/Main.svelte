<script>
	let { children } = $props();
</script>

<main class="flex flex-1 flex-col overflow-auto">
	{@render children()}
</main>
