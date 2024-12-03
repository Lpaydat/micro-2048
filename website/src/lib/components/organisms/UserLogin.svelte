<script lang="ts">
	import { userStore } from '$lib/stores/userStore';
	import JoinForm from '../molecules/JoinForm.svelte';
	import UserDetails from './UserDetails.svelte';
</script>

<div class="flex h-full w-full items-end">
	{#if $userStore.username}
		<UserDetails />
	{:else}
		<div class="mx-auto mb-16 flex items-center justify-center">
			<JoinForm />
		</div>
	{/if}
</div>
