<script>
	import BrandLogo from '../atoms/BrandLogo.svelte';

	let { children } = $props();
</script>

<div class="m-2 flex items-center justify-between rounded-lg bg-[#faf8ef] px-4 py-2 shadow-sm">
	<a href="/">
		<div class="flex-none">
			<BrandLogo />
		</div>
	</a>

	<div class="flex flex-1 justify-end">
		{@render children?.()}
	</div>
</div>
