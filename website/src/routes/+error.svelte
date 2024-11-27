<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
</script>

<div
	class="flex min-h-screen items-center justify-center bg-[#23232b] bg-[url('/micro-carbon.png')] bg-repeat"
>
	<div
		class="rounded-2xl border border-[#3b3b41] bg-[#23232b]/50 p-8 text-center shadow-xl backdrop-blur-sm"
	>
		<!-- Game-style tile with error number -->
		<div
			class="mx-auto mb-6 flex h-32 w-32 transform items-center justify-center rounded-xl bg-[#edc22e] shadow-lg transition-transform hover:scale-105"
		>
			<span class="text-5xl font-bold text-white">{$page.status}</span>
		</div>

		<!-- Error message -->
		<h1 class="mb-4 text-3xl font-bold text-white">
			{#if $page.status === 404}
				Page Not Found
			{:else}
				{$page.error?.message || 'An error occurred'}
			{/if}
		</h1>

		<!-- Custom message based on error -->
		<p class="mb-8 text-gray-300">
			{#if $page.status === 404}
				Oops! Looks like this tile slid off the board.
			{:else}
				Something went wrong while merging tiles.
			{/if}
		</p>

		<!-- Home button styled like a 2048 tile -->
		<button
			on:click={() => goto('/')}
			class="transform rounded-lg bg-[#edc22e] px-8 py-3 font-bold text-white shadow-lg transition-all hover:scale-105 hover:bg-[#edcc60]"
		>
			Back to Game
		</button>
	</div>
</div>
