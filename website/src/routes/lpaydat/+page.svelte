<script lang="ts">
	import { goto } from '$app/navigation';
	import { toggleMod } from '$lib/graphql/mutations';
	import { getPlayerInfo } from '$lib/graphql/queries/getPlayerInfo';
	import { getContextClient } from '@urql/svelte';
	import { onMount } from 'svelte';

	let isReady = $state(false);
	let username = $state('lpaydat');
	const client = getContextClient();

	const data = $derived(getPlayerInfo(client, username));

	const toggleModerator = () => {
		const modUsername = username.trim();
		if (modUsername === '') {
			return;
		}
		toggleMod(client, modUsername);
		data.reexecute({ variables: { username: modUsername } });
	};

	onMount(() => {
		const username = localStorage.getItem('username');
		if (username !== 'lpaydat') {
			goto('/error');
		}
		setTimeout(() => {
			isReady = true;
		}, 1000);
	});
</script>

{#if isReady}
	<div class="flex h-screen w-full flex-col items-center justify-center gap-4 bg-surface-800">
		<input type="text" class="p-2" bind:value={username} />
		<div class="flex gap-2">
			<button
				type="button"
				class="variant-filled-primary btn rounded-sm px-2 py-1 text-sm font-bold"
				onclick={toggleModerator}
			>
				CLICK
			</button>
		</div>
		<div class="flex flex-col items-center justify-center gap-2 font-semibold text-white">
			{#if $data.fetching}
				<p>Loading player info...</p>
			{:else if $data.data?.player}
				<p class="text-lg font-bold">Username: {$data.data.player.username}</p>
				<p class="text-lg font-bold">Moderator Status: {$data.data.player.isMod ? 'Yes' : 'No'}</p>
			{:else}
				<p>Player not found</p>
			{/if}
		</div>
	</div>
{/if}
