<script lang="ts">
    export let icon: string = '';
    export let label: string = '';
    export let disabled: boolean = false;
    export let color: 'default' | 'important' | 'warning' | 'danger' | 'disabled' = 'default';
    export let hoverColor: 'default' | 'important' | 'warning' | 'danger' | 'disabled' = 'default';
    export let loading: boolean = false;

    const baseColorClasses = {
        default: 'bg-orange-500/50',
        important: 'text-orange-600 variant-outline-primary hover:text-white',
        warning: 'bg-warning-500',
        danger: 'bg-error-500',
        disabled: 'bg-surface-300-600-token text-surface-600-300-token cursor-not-allowed'
    };

    const hoverColorClasses = {
        default: 'hover:bg-orange-500 hover:text-black',
        important: 'hover:bg-orange-600',
        warning: 'hover:bg-warning-600',
        danger: 'hover:bg-error-800',
        disabled: ''
    };

    $: colorClass = baseColorClasses[disabled ? 'disabled' : color];
    $: hoverClass = !disabled && !loading ? hoverColorClasses[hoverColor || color] : '';
    $: loadingClass = loading ? 'cursor-not-allowed text-surface-400' : '';
</script>

<button 
    class="flex items-center justify-center rounded-lg px-2 lg:px-4 py-2 text-xs lg:text-sm font-bold transition-all {colorClass} {hoverClass} {loading ? 'cursor-not-allowed' : ''}"
    on:click
    disabled={disabled || loading}
>
    {#if loading}
        <div class="loading-spinner mr-2"></div>
    {:else if icon}
        <i class="fas fa-{icon}"></i>
    {/if}
    <span class="tracking-wider {loadingClass}">{label}</span>
</button> 

<style>
    .loading-spinner {
        border: 4px solid rgba(255, 255, 255, 0.3);
        border-top: 4px solid #fff;
        border-radius: 50%;
        width: 16px;
        height: 16px;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        0% { transform: rotate(0deg); }
        100% { transform: rotate(360deg); }
    }
</style>