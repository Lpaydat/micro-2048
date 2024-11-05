<script lang="ts">
    export let icon: string = '';
    export let label: string = '';
    export let disabled: boolean = false;
    export let color: 'default' | 'warning' | 'danger' | 'disabled' = 'default';
    export let hoverColor: 'default' | 'warning' | 'danger' | 'disabled' = 'default';

    const baseColorClasses = {
        default: 'bg-white/20',
        warning: 'bg-warning-500',
        danger: 'bg-error-500',
        disabled: 'bg-surface-300-600-token text-surface-600-300-token cursor-not-allowed'
    };

    const hoverColorClasses = {
        default: 'hover:bg-orange-500',
        warning: 'hover:bg-warning-600',
        danger: 'hover:bg-error-800',
        disabled: ''
    };

    $: colorClass = baseColorClasses[color];
    $: hoverClass = !disabled ? hoverColorClasses[hoverColor] : '';
</script>

<button 
    class="flex items-center rounded-lg px-4 py-2 text-sm font-bold transition-all {colorClass} {hoverClass} {!disabled && 'hover:shadow-md'}"
    on:click
    {disabled}
>
    {#if icon}
        <i class="fas fa-{icon}"></i>
    {/if}
    <span class="tracking-wider">{label}</span>
</button> 