<script>
	import BrandLogo from "../atoms/BrandLogo.svelte";
</script>

<div class="flex justify-between m-2 rounded-lg items-center px-4 py-3 bg-[#faf8ef] shadow-sm">
    <div class="flex-none">
        <BrandLogo />
    </div>

    <div class="flex-1 flex justify-end">
        <slot />
    </div>
</div>