<script>
	let { children } = $props();
</script>

<div class="z-10 flex h-screen w-96 min-w-[25%] flex-col justify-between overflow-hidden py-8">
	{@render children?.()}
</div>
