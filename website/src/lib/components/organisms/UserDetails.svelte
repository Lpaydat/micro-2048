<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { userStore } from '$lib/stores/userStore';
	import Button from '../atoms/Button.svelte';
	import { getModalStore } from '@skeletonlabs/skeleton';
	import type { ModalSettings, ModalStore } from '@skeletonlabs/skeleton';
	import { logout } from '$lib/utils/logout';
	import { getClient } from '$lib/client';
	import { getPlayerInfo } from '$lib/graphql/queries/getPlayerInfo';
	import { onMount } from 'svelte';
	import { formatBalance } from '$lib/utils/formatBalance';
	import { requestFaucetMutation } from '$lib/graphql/mutations/requestFaucet';

	const modalStore: ModalStore = getModalStore();

	const playerClient = $derived(getClient($userStore.chainId));
	const playerInfo = $derived(getPlayerInfo(playerClient, $userStore.username ?? ''));

	$effect(() => {
		localStorage.setItem('username', $userStore.username ?? '');
		localStorage.setItem('passwordHash', $userStore.passwordHash ?? '');
		localStorage.setItem('chainId', $userStore.chainId ?? '');
	});

	const handleLogout = () => {
		logout();

		if ($page.url.pathname === '/game') {
			goto('/');
		}
	};

	const isElimination = $page.url.pathname.includes('elimination');

	const howToPlayModal: ModalSettings = {
		type: 'component',
		component: isElimination ? 'how-to-play-elimination' : 'how-to-play-2048'
	};

	const howToPlay = () => {
		modalStore.trigger(howToPlayModal);
	};

	let fetchInterval: NodeJS.Timeout;
	onMount(() => {
		playerInfo.reexecute({ requestPolicy: 'network-only' });

		fetchInterval = setInterval(() => {
			playerInfo.reexecute({ requestPolicy: 'network-only' });
			localStorage.setItem('balance', $playerInfo?.data?.balance ?? '0.00');
		}, 2000);

		return () => clearInterval(fetchInterval);
	});
</script>

<div
	class="mx-auto w-80 max-w-md rounded-md border-[8px] border-[#BBADA0] bg-[#FAF8EF] p-6 shadow-md"
>
	<div class="space-y-4">
		<!-- Title -->
		<div class="text-center">
			<h2 class="game-font text-2xl font-bold text-[#776E65]">Game Profile</h2>
		</div>

		<!-- User Info Box -->
		<div class="w-full rounded-lg bg-[#bbada0] p-4 shadow-md">
			<p class="game-font mb-1 text-center text-sm font-bold text-[#eee4da]">LOGGED IN AS</p>
			<p class="game-font text-center text-xl font-bold tracking-wide text-[#f9f6f2]">
				{$userStore.username}
			</p>
		</div>

		<!-- Balance Section -->
		<div class="flex items-center justify-between rounded-lg bg-[#bbada0] px-4 py-3">
			<div>
				<p class="game-font mb-1 text-xs font-bold uppercase tracking-wider text-[#fffefc]">
					Balance
				</p>
				<p class="game-font text-lg font-bold text-cyan-800">
					{formatBalance($playerInfo?.data?.balance ?? '0.00')}
				</p>
			</div>
			<Button
				size="sm"
				class="sm btn text-black"
				variant="primary"
				onclick={() => requestFaucetMutation(playerClient)}
				disabled={parseFloat($playerInfo?.data?.balance ?? '0.00') > 0.2}
			>
				Faucet
			</Button>
		</div>

		<!-- Actions -->
		<div class="flex items-center justify-between border-t border-[#CDC1B4] pt-4">
			<Button variant="outline" size="sm" onclick={howToPlay}>How to Play</Button>

			<Button variant="default" size="sm" onclick={handleLogout}>Logout</Button>
		</div>
	</div>
</div>

<style>
	:global(.game-font) {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
		-webkit-font-smoothing: antialiased;
	}

	/* Responsive padding */
	/* @media (max-width: 640px) {
		div.container {
			border-width: 4px;
			margin: 0 1rem;
		}
	} */
</style>
