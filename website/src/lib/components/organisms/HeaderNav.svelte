<script lang="ts">
	import NavLink from "../atoms/NavLink.svelte";
	import { popup } from '@skeletonlabs/skeleton';
	import type { PopupSettings } from '@skeletonlabs/skeleton';
	import { onMount } from 'svelte';
	
	let isMobile = false;
	
	const popupSettings: PopupSettings = {
		event: 'click',
		target: 'popupSettings',
		placement: 'bottom-end',
		middleware: {
			offset: 8
		}
	};
	
	onMount(() => {
		checkMobile();
		window.addEventListener('resize', checkMobile);
		return () => window.removeEventListener('resize', checkMobile);
	});
	
	function checkMobile() {
		isMobile = window.innerWidth < 768;
	}
</script>

<div class="nav-container mt-3 lg:mt-5 mx-4 lg:me-5 z-10">
	{#if isMobile}
		<!-- Mobile Menu Button with Popup -->
		<div class="float-right">
			<button 
				aria-label="Menu"
				class="text-red-500 p-2"
				use:popup={popupSettings}
			>
				<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
				</svg>
			</button>
			
			<!-- Mobile Menu Popup -->
			<div class="card p-4 w-48 shadow-xl" data-popup="popupSettings">
				<nav class="list-nav">
					<ul class="flex flex-col gap-2">
						<li><NavLink href="https://linera.io" target="_blank" rel="noopener noreferrer">Linera.io</NavLink></li>
						<li><NavLink href="https://linera.dev" target="_blank" rel="noopener noreferrer">Documentation</NavLink></li>
						<li><NavLink href="https://discord.gg/linera" target="_blank" rel="noopener noreferrer">Discord</NavLink></li>
						<li><NavLink href="https://t.me/linera" target="_blank" rel="noopener noreferrer">Telegram</NavLink></li>
					</ul>
				</nav>
			</div>
		</div>
	{:else}
		<!-- Desktop Navigation -->
		<ul class="flex justify-end gap-8">
			<li><NavLink href="https://linera.io" target="_blank" rel="noopener noreferrer">Linera.io</NavLink></li>
			<li><NavLink href="https://linera.dev" target="_blank" rel="noopener noreferrer">Documentation</NavLink></li>
			<li><NavLink href="https://discord.gg/linera" target="_blank" rel="noopener noreferrer">Discord</NavLink></li>
			<li><NavLink href="https://t.me/linera" target="_blank" rel="noopener noreferrer">Telegram</NavLink></li>
		</ul>
	{/if}
</div>
