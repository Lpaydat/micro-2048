<script>
	let { children } = $props();
</script>

<div class="z-10 flex h-full min-h-screen w-96 min-w-[25%] flex-col justify-between py-8">
	{@render children?.()}
</div>
