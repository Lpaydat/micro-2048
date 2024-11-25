<script lang="ts">
    export let title: string;
    export let prevPage: string | undefined = undefined;
    export let color: 'green' | 'orange' | 'red' | 'blue' | 'purple' | 'yellow' | 'pink' | 'gray' | 'teal' | 'cyan' = 'green';

    $: headerRound = !prevPage ? 'md:rounded-bl-lg mb-4 md:mb-0' : '';

    const colorClass = {
        green: 'bg-green-600',
        orange: 'bg-orange-600',
        red: 'bg-red-600',
        blue: 'bg-blue-600',
        purple: 'bg-purple-600',
        yellow: 'bg-yellow-600',
        pink: 'bg-pink-600',
        gray: 'bg-gray-600',
        teal: 'bg-teal-600',
        cyan: 'bg-cyan-600'
    }
</script>

<div class="flex w-full flex-col">
    <div class="font-bold w-full flex items-center bg-gradient-to-r from-warning-500 to-warning-600 text-white {headerRound} p-2 md:p-4 shadow-md transition-all relative">
        <div
            class="header flex items-center justify-center gap-2 lg:gap-4 rounded-lg px-2 lg:px-4 py-2 text-xs lg:text-sm font-bold transition-all {colorClass[color]}"
        >
            {#if $$slots.icon}
                <slot name="icon" />
            {/if}
            <span class="tracking-wider text-md lg:text-2xl">{title}</span>
        </div>
        
        <div class="ms-auto flex items-center gap-2">
            {#if $$slots.actions}
                <slot name="actions" />
            {/if}
        </div>
    </div>

    {#if prevPage}
        <div class="flex">
            <a href={prevPage} class="inline-block text-xs lg:text-sm px-3 py-2 text-white rounded-br-md lg:rounded-b-md bg-surface-400 hover:bg-surface-800 transition-colors">
                Back
            </a>
        </div>
    {/if}
</div>

<style>
    .header {
        border: 2px solid #fff;
        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }
</style>