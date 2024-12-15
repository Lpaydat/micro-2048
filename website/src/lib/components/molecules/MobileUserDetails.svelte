<script>
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { userStore } from '$lib/stores/userStore';
	import LogOut from 'lucide-svelte/icons/log-out';
	import { logout } from '$lib/utils/logout';
	import JoinForm from './JoinForm.svelte';

	const handleLogout = () => {
		logout();

		if ($page.url.pathname === '/game') {
			goto('/');
		}
	};
</script>

<div class="flex items-center gap-3">
	{#if $userStore.username}
		<span class="text-md font-bold text-[#776e65]">{$userStore.username}</span>
		<button
			type="button"
			onclick={handleLogout}
			class="rounded-[3px] bg-[#8f7a66] p-2 text-sm font-bold text-white transition-colors hover:bg-[#7f6a56]"
		>
			<LogOut size={16} />
		</button>
	{:else}
		<JoinForm mobileForm />
	{/if}
</div>
