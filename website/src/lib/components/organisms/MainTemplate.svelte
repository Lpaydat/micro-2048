<script lang="ts">
	import { onMount, type Snippet } from 'svelte';
	import Main from '../molecules/Main.svelte';
	import MobileHeader from '../molecules/MobileHeader.svelte';
	import Sidebar from '../molecules/Sidebar.svelte';
	import MobileUserDetails from '../molecules/MobileUserDetails.svelte';
	import Credits from '../molecules/Credits.svelte';
	import { isMobile as isMobileStore } from '$lib/stores/isMobile';

	interface Props {
		windowWidth?: number;
		windowHeight?: number;
		mainCenter?: boolean;
		overflowHidden?: boolean;
		hideCredits?: boolean;
		sidebar?: Snippet;
		header?: Snippet;
		subHeader?: Snippet;
		main?: Snippet;
		footer?: Snippet;
	}

	let {
		windowWidth = $bindable(0),
		windowHeight = $bindable(0),
		mainCenter,
		overflowHidden,
		hideCredits = false,
		sidebar,
		header,
		subHeader,
		main,
		footer
	}: Props = $props();

	onMount(() => {
		const updateDimensions = () => {
			windowWidth = window.innerWidth;
			windowHeight = window.innerHeight;
		};
		window.addEventListener('resize', updateDimensions);
		updateDimensions();
		return () => window.removeEventListener('resize', updateDimensions);
	});

	const isMobile = $derived(windowWidth <= 820 || windowWidth <= windowHeight);
	const mainClass = $derived(mainCenter ? 'justify-center' : 'justify-start');
	const overflowClass = $derived(overflowHidden ? 'overflow-hidden' : '');

	$effect(() => {
		isMobileStore.set(windowWidth <= 820 || windowWidth <= windowHeight);
	});
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
					{@render subHeader?.()}
				</div>
			{/if}

			<div class="flex flex-1 items-center {mainClass} flex-col">
				{@render main?.()}
			</div>

			<div class="flex-none">
				{@render footer?.()}
				{#if !hideCredits}
					<Credits />
				{/if}
			</div>
		</div>
	</Main>
</div>
