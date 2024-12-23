<script lang="ts">
	let { description = 'Default game mode description' }: { description?: string } = $props();
</script>

<div
	class="description-box custom-shadow custom-transition w-full scale-110 transform rounded-lg border-4 border-surface-800 bg-surface-600 p-5 text-center font-sans text-lg text-white"
>
	{description}
</div>

<style>
	.custom-shadow {
		box-shadow: 0 4px 8px rgba(0, 0, 0, 0.2); /* Subtle shadow for depth */
	}
	.custom-transition {
		transition: transform 0.2s;
	}
</style>
