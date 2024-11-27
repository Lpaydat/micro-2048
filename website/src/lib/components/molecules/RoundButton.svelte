<script lang="ts">
	import { goto } from '$app/navigation';

	export let isRoundEnded: boolean;
	export let countdown: number;
	export let status: string;
</script>

<div class="lg:h-18 flex h-12 items-center justify-center pt-4 lg:pt-8 xl:h-32 xl:pt-10">
	{#if !isRoundEnded}
		<div class="game-font text-center text-5xl font-bold text-[#776E65] lg:text-6xl xl:text-7xl">
			{countdown >= 0 ? countdown : '...'}
		</div>
	{:else if status === 'Ended'}
		<button
			class="game-font variant-soft-surface btn text-center text-xl font-bold text-[#F67C5F] lg:text-2xl xl:text-3xl"
			on:click={() => goto('/elimination')}
		>
			Lobby
		</button>
	{:else}
		<button
			class="game-font variant-soft-surface btn text-center text-xl font-bold text-[#F67C5F] lg:text-2xl xl:text-3xl"
			on:click
		>
			Next Round
		</button>
	{/if}
</div>
