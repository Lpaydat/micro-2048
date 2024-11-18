<script lang="ts">
    export let title: string;
    export let prevPage: string | undefined = undefined;

    $: headerRound = !prevPage ? 'md:rounded-bl-lg mb-4 md:mb-0' : '';
</script>

<div class="h2 font-bold flex items-center gap-4 bg-gradient-to-r from-warning-500 to-warning-600 text-white {headerRound} p-4 shadow-md transition-all relative">
    {#if $$slots.icon}
        <slot name="icon" />
    {/if}
    <span class="tracking-wider">{title}</span>
    {#if $$slots.actions}
        <slot name="actions" />
    {/if}
</div>

{#if prevPage}
    <a href={prevPage} class="inline-block text-sm px-3 py-2 text-white rounded-t-none rounded-md bg-surface-400 card hover:bg-surface-800 transition-colors">
        Back
    </a>
{/if}
