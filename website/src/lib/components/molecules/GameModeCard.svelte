<script lang="ts">
	export let disabled = false;
</script>

<div
	role="button"
	class="card variant-filled-surface p-4 transition-all duration-200 {!disabled &&
		'cursor-pointer hover:scale-105 hover:shadow-xl'} flex h-full w-full flex-col rounded-lg"
	class:opacity-50={disabled}
	tabindex={disabled ? -1 : 0}
	on:click
	on:keydown
>
	<header class="card-header pb-4 text-center text-2xl font-bold">
		<slot name="title" />
	</header>

	<section class="flex-grow p-4">
		<slot name="description" />
	</section>

	<section class="pt-4">
		<slot name="actions" />
	</section>
</div>
