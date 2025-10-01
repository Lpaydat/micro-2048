<script lang="ts">
	import BrandLogo from '../atoms/BrandLogo.svelte';
</script>

<div class="flex justify-center">
	<a href="/" class="block">
		<div class="mx-auto w-80 max-w-md rounded-lg bg-[#faf8ef] p-6 text-center shadow-md">
			<BrandLogo />
			<div class="relative">
				<h2 class="relative text-xl font-normal text-[#776e65]">
					on <span class="font-semibold text-[#cc9900]">microchains</span>
				</h2>
				<span
					class="font-handwriting absolute right-6 -top-12 z-10 -rotate-12 transform text-2xl text-red-500 opacity-80 drop-shadow-[0_1px_1px_rgba(0,0,0,0.25)]"
				>
					Conway
					<!-- Beta -->
				</span>
				<div
					class="absolute -bottom-3 left-0 right-0 h-1 rounded-full bg-[#bbada0] opacity-30"
				></div>
			</div>
		</div>
	</a>
</div>

<style>
	@import url('https://fonts.googleapis.com/css2?family=Dancing+Script:wght@600&display=swap');

	div {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
	}

	.font-handwriting {
		font-family: 'Dancing Script', cursive;
		text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
	}
</style>
