<script lang="ts">
	import { page } from '$app/stores';
	import BoardReplay from '$lib/components/organisms/BoardReplay.svelte';
	import { goto } from '$app/navigation';
	
	const boardId = $derived($page.url.searchParams.get('boardId'));
	const chainId = $derived($page.url.searchParams.get('chainId'));

	$effect(() => {
		if (!boardId || !chainId) {
			goto('/');
		}
	});
</script>

<svelte:head>
	<title>Replay - 2048</title>
</svelte:head>

<div class="flex min-h-screen flex-col items-center justify-center bg-[#faf8ef] p-4">
	<div class="mb-4">
		<h1 class="text-center text-4xl font-bold text-[#776e65]">Game Replay</h1>
		<p class="text-center text-sm text-surface-500">Watch how this game was played</p>
	</div>

	{#if boardId && chainId}
		<BoardReplay {boardId} {chainId} />
	{:else}
		<div class="text-red-500">Missing boardId or chainId parameter</div>
	{/if}

	<div class="mt-4">
		<a
			href="/leaderboard"
			class="rounded-md bg-surface-700 px-4 py-2 text-sm font-bold text-white transition-colors hover:bg-surface-600"
		>
			← Back to Leaderboard
		</a>
	</div>
</div>
