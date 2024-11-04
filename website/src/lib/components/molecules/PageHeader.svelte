<script lang="ts">
    export let title: string;
</script>

<div class="h2 mb-4 font-bold flex items-center gap-4 bg-gradient-to-r from-warning-500 to-warning-600 text-white rounded-bl-lg p-4 shadow-md transition-all">
    {#if $$slots.icon}
        <slot name="icon" />
    {/if}
    <span class="tracking-wider">{title}</span>
    {#if $$slots.actions}
        <slot name="actions" />
    {/if}
</div>
