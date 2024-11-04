<script lang="ts">
    export let icon: string = '';
    export let label: string = '';
    export let disabled: boolean = false;
    export let color: 'default' | 'warning' | 'disabled' = 'default';

    const colorClasses = {
        default: 'bg-white/20 hover:bg-orange-500',
        warning: 'bg-warning-500 hover:bg-warning-600',
        disabled: 'bg-surface-300-600-token text-surface-600-300-token cursor-not-allowed'
    };
</script>

<button 
    class="flex items-center gap-2 rounded-lg px-4 py-2 text-sm font-bold transition-all {colorClasses[color]} {!disabled && 'hover:shadow-md'}"
    on:click
    {disabled}
>
    {#if icon}
        <i class="fas fa-{icon}"></i>
    {/if}
    <span class="tracking-wider">{label}</span>
</button> 