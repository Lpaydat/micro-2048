<script lang="ts">
	import { goto } from '$app/navigation';
	import { toggleMod } from '$lib/graphql/mutations';
	import { getContextClient } from '@urql/svelte';
	import { onMount } from 'svelte';

	let isReady = $state(false);
	let username = $state('');
	const client = getContextClient();

	const toggleModerator = () => {
		const modUsername = username.trim();
		if (modUsername === '') {
			return;
		}
		toggleMod(client, modUsername);
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
		<input type="text" bind:value={username} />
		<div class="flex gap-2">
			<button
				type="button"
				class="variant-filled-primary btn rounded-sm px-2 py-1 text-sm font-bold"
				onclick={toggleModerator}
			>
				CLICK
			</button>
		</div>
	</div>
{/if}
