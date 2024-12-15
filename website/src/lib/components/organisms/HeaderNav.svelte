<script lang="ts">
	import { isMobile } from '$lib/stores/isMobile';
	import NavLink from '../atoms/NavLink.svelte';
	import { popup } from '@skeletonlabs/skeleton';
	import type { PopupSettings } from '@skeletonlabs/skeleton';

	const popupSettings: PopupSettings = {
		event: 'click',
		target: 'popupSettings',
		placement: 'bottom-end',
		middleware: {
			offset: 8
		}
	};
</script>

<div class="nav-container z-10 mx-4 mt-3 lg:me-5 lg:mt-5">
	{#if $isMobile}
		<!-- Mobile Menu Button with Popup -->
		<div class="float-right">
			<button aria-label="Menu" class="p-2 text-red-500" use:popup={popupSettings}>
				<svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						stroke-width="2"
						d="M4 6h16M4 12h16M4 18h16"
					/>
				</svg>
			</button>

			<!-- Mobile Menu Popup -->
			<div class="card w-48 p-4 shadow-xl" data-popup="popupSettings">
				<nav class="list-nav">
					<ul class="flex flex-col gap-2">
						<li>
							<NavLink href="https://linera.io" target="_blank" rel="noopener noreferrer"
								>Linera.io</NavLink
							>
						</li>
						<li>
							<NavLink href="https://linera.dev" target="_blank" rel="noopener noreferrer"
								>Documentation</NavLink
							>
						</li>
						<li>
							<NavLink href="https://discord.gg/linera" target="_blank" rel="noopener noreferrer"
								>Discord</NavLink
							>
						</li>
						<li>
							<NavLink href="https://t.me/linera" target="_blank" rel="noopener noreferrer"
								>Telegram</NavLink
							>
						</li>
					</ul>
				</nav>
			</div>
		</div>
	{:else}
		<!-- Desktop Navigation -->
		<ul class="flex justify-end gap-8">
			<li>
				<NavLink href="https://linera.io" target="_blank" rel="noopener noreferrer"
					>Linera.io</NavLink
				>
			</li>
			<li>
				<NavLink href="https://linera.dev" target="_blank" rel="noopener noreferrer"
					>Documentation</NavLink
				>
			</li>
			<li>
				<NavLink href="https://discord.gg/linera" target="_blank" rel="noopener noreferrer"
					>Discord</NavLink
				>
			</li>
			<li>
				<NavLink href="https://t.me/linera" target="_blank" rel="noopener noreferrer"
					>Telegram</NavLink
				>
			</li>
		</ul>
	{/if}
</div>
