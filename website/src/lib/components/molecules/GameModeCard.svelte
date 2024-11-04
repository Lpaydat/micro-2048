<script lang="ts">
    export let disabled = false;
</script>

<div 
    role="button"
    class="card variant-filled-surface p-4 transition-all duration-200 {!disabled && 'hover:scale-105 hover:shadow-xl cursor-pointer'} w-full h-full flex flex-col rounded-lg"
    class:opacity-50={disabled}
    tabindex={disabled ? -1 : 0}
    on:click
    on:keydown
>
    <header class="card-header text-2xl font-bold pb-4 text-center">
        <slot name="title" />
    </header>
    
    <section class="p-4 flex-grow">
        <slot name="description" />
    </section>

    <section class="pt-4">
        <slot name="actions" />
    </section>
</div>
