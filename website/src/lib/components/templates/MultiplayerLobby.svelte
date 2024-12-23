<script lang="ts">
	import { getModalStore } from '@skeletonlabs/skeleton';
	import Swords from 'lucide-svelte/icons/swords';
	import HelpCircle from 'lucide-svelte/icons/circle-help';

	import ActionButton from '../atoms/ActionButton.svelte';
	import PageHeader from '../molecules/PageHeader.svelte';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import WaitingGames from '../organisms/WaitingGames.svelte';
	import EliminationGameForm from '../organisms/EliminationGameForm.svelte';
	import type { ModalSettings, ModalStore } from '@skeletonlabs/skeleton';
	import HelpButton from '../atoms/HelpButton.svelte';

	const modalStore: ModalStore = getModalStore();

	const hostGameModal: ModalSettings = {
		type: 'component',
		component: { ref: EliminationGameForm }
	};

	const howToPlayModal: ModalSettings = {
		type: 'component',
		component: 'how-to-play-elimination'
	};

	const hostGame = () => {
		modalStore.trigger(hostGameModal);
	};

	const howToPlay = () => {
		modalStore.trigger(howToPlayModal);
	};
</script>

<MainTemplate>
	{#snippet sidebar()}
		<UserSidebar />
	{/snippet}

	{#snippet main()}
		<PageHeader color="cyan" title="ELIMINATION GAME" prevPage="/">
			{#snippet icon()}
				<Swords size={28} />
			{/snippet}
			{#snippet actions()}
				<HelpButton ariaLabel="How to Play" onclick={howToPlay}>
					<HelpCircle size={20} />
				</HelpButton>
				<ActionButton label="HOST GAME" onclick={hostGame} />
			{/snippet}
		</PageHeader>
		<WaitingGames />
	{/snippet}
</MainTemplate>
