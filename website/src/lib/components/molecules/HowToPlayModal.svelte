<script lang="ts">
	import { getModalStore } from '@skeletonlabs/skeleton';

	export let title = 'How to Play';
	export let items: { text: string }[] = [];

	const modalStore = getModalStore();
</script>

<div class="w-full max-w-lg rounded-lg bg-white shadow-xl">
	<!-- Header -->
	<div class="border-b p-3 pb-2 sm:p-4 sm:pb-3 md:p-6 md:pb-4">
		<h2 class="text-lg font-bold text-gray-800 sm:text-xl md:text-2xl">{title}</h2>
	</div>

	<!-- Content -->
	<div class="space-y-3 p-3 sm:space-y-4 sm:p-4 md:space-y-6 md:p-6">
		<div class="space-y-2 sm:space-y-3 md:space-y-4">
			{#each items as item, i}
				<div class="flex items-start gap-2 sm:gap-3 md:gap-4">
					<div
						class="flex h-6 w-6 flex-shrink-0 items-center justify-center rounded-full bg-warning-100 text-sm font-medium text-warning-700 sm:h-7 sm:w-7 sm:text-base md:h-8 md:w-8"
					>
						{i + 1}
					</div>
					<p class="pt-0.5 text-sm text-gray-700 sm:pt-1 sm:text-base md:text-lg">
						{item.text}
					</p>
				</div>
			{/each}
		</div>
	</div>

	<!-- Footer -->
	<div class="p-3 pt-0 sm:p-4 md:p-6">
		<button
			class="w-full rounded-md bg-warning-500 py-2 text-sm font-medium text-white transition-colors hover:bg-warning-600 sm:py-2.5 sm:text-base md:py-3 md:text-lg"
			on:click={() => modalStore.close()}
		>
			Got it!
		</button>
	</div>
</div>
