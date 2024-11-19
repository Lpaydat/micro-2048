<script lang="ts">
    export let ariaLabel: string;
</script>

<button 
    class="w-8 h-8 flex items-center justify-center bg-white/20 hover:bg-white/30 rounded-md transition-colors"
    aria-label={ariaLabel}
    on:click
>
    <slot />
</button>
