<script>
	import BrandLogo from '../atoms/BrandLogo.svelte';

	let { children } = $props();
</script>

<div class="m-2 flex items-center justify-between rounded-lg bg-[#faf8ef] px-4 py-2 shadow-sm">
	<a href="/">
		<div class="relative flex-none">
			<BrandLogo />
			<span
				class="font-handwriting text-md absolute -bottom-1 left-8 z-10 -rotate-12 transform text-red-500 opacity-80 drop-shadow-[0_1px_1px_rgba(0,0,0,0.25)]"
			>
				Archimedes
			</span>
		</div>
	</a>

	<div class="flex flex-1 justify-end">
		{@render children?.()}
	</div>
</div>

<style>
	@import url('https://fonts.googleapis.com/css2?family=Dancing+Script:wght@600&display=swap');

	.font-handwriting {
		font-family: 'Dancing Script', cursive;
		text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}
</style>
