<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import Main from '../molecules/Main.svelte';
	import MobileHeader from '../molecules/MobileHeader.svelte';
	import Sidebar from '../molecules/Sidebar.svelte';
	import MobileUserDetails from '../molecules/MobileUserDetails.svelte';

	interface Props {
		windowWidth?: number;
		mainCenter?: boolean;
		overflowHidden?: boolean;
		sidebar?: Snippet;
		header?: Snippet;
		main?: Snippet;
		footer?: Snippet;
	}

	let {
		windowWidth = $bindable(0),
		mainCenter,
		overflowHidden,
		sidebar,
		header,
		main,
		footer
	}: Props = $props();

	onMount(() => {
		const updateWidth = () => (windowWidth = window.innerWidth);
		window.addEventListener('resize', updateWidth);
		updateWidth();
		return () => window.removeEventListener('resize', updateWidth);
	});

	const isMobile = $derived(windowWidth <= 768);
	const mainClass = $derived(mainCenter ? 'justify-center' : 'justify-start');
	const overflowClass = $derived(overflowHidden ? 'overflow-hidden' : '');
</script>

<div class="flex h-screen {overflowClass} bg-[#23232b] bg-[url('/micro-carbon.png')] bg-repeat">
	{#if !isMobile}
		<Sidebar>
			{@render sidebar?.()}
		</Sidebar>
	{/if}

	<Main>
		<div class="flex flex-1 flex-col {overflowClass}">
			{#if isMobile}
				<div class="flex-none">
					<MobileHeader>
						{#if header}
							{@render header?.()}
						{:else}
							<MobileUserDetails />
						{/if}
					</MobileHeader>
				</div>
			{/if}

			<div class="flex flex-1 items-center {mainClass} flex-col">
				{@render main?.()}
			</div>

			<div class="flex-none">
				{@render footer?.()}
			</div>
		</div>
	</Main>
</div>
