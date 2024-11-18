<script>
	import { userStore } from '$lib/stores/userStore';
    import LogOut from 'lucide-svelte/icons/log-out';
	import MobileUserLogin from './MobileUserLogin.svelte';

    const handleLogout = () => {
        $userStore.chainId = undefined;
        $userStore.username = undefined;
        $userStore.passwordHash = undefined;
        localStorage.removeItem('username');
        localStorage.removeItem('passwordHash');
        localStorage.removeItem('chainId');
    }
</script>

<div class="flex items-center gap-3">
    {#if $userStore.username}
        <span class="text-[#776e65] font-bold text-md">{$userStore.username}</span>
        <button
            type="button"
            on:click={handleLogout}
            class="bg-[#8f7a66] text-white text-sm font-bold p-2 rounded-[3px] hover:bg-[#7f6a56] transition-colors"
        >
            <LogOut size={16} />
        </button>
    {:else}
        <MobileUserLogin />
    {/if}
</div>
