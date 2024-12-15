<script lang="ts">
	import { isMobile } from '$lib/stores/isMobile';
</script>

<h1
	class="mb-1 text-[32px] {!$isMobile ? 'lg:text-[80px]' : ''} font-bold leading-tight tracking-tight text-[#776e65] lg:mb-2"
>
	2048
</h1>

<style>
	h1 {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
		text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.1);
	}
</style>
