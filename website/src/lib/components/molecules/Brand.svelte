<script>
	import BrandLogo from '../atoms/BrandLogo.svelte';
</script>

<div class="flex justify-center">
	<a href="/" class="block">
		<div class="mx-auto w-80 max-w-md rounded-lg bg-[#faf8ef] p-6 text-center shadow-md">
			<BrandLogo />
			<div class="relative">
				<h2 class="relative text-xl font-normal text-[#776e65]">
					on <span class="font-semibold text-[#edc403]">microchain</span>
				</h2>
				<div
					class="absolute -bottom-3 left-0 right-0 h-1 rounded-full bg-[#bbada0] opacity-30"
				></div>
			</div>
		</div>
	</a>
</div>

<style>
	div {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
	}
</style>
