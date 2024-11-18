<script>
    import BrandLogo from '../atoms/BrandLogo.svelte';
</script>

<div class="flex justify-center">
    <a href="/" class="block">
        <div class="text-center p-6 w-80 rounded-lg bg-[#faf8ef] shadow-md max-w-md mx-auto">
            <BrandLogo />
            <div class="relative">
                <h2 class="text-xl text-[#776e65] font-normal relative">
                    on <span class="font-semibold text-[#edc403]">microchain</span>
                </h2>
                <div class="absolute -bottom-3 left-0 right-0 h-1 bg-[#bbada0] rounded-full opacity-30"></div>
            </div>
        </div>
    </a>
</div>

<style>
    div {
        font-family: "Clear Sans", "Helvetica Neue", Arial, sans-serif;
    }
</style>
