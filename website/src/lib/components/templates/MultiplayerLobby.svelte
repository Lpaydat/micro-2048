<script lang="ts">
    import { getModalStore } from '@skeletonlabs/skeleton';
    import Swords from 'lucide-svelte/icons/swords';
    import HelpCircle from 'lucide-svelte/icons/circle-help';

	import ActionButton from '../atoms/ActionButton.svelte';
	import PageHeader from "../molecules/PageHeader.svelte";
    import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import WaitingGames from "../organisms/WaitingGames.svelte";
    import EliminationGameForm from '../organisms/EliminationGameForm.svelte';
	import type { ModalSettings, ModalStore } from '@skeletonlabs/skeleton';
	import HelpButton from '../atoms/HelpButton.svelte';

    const modalStore: ModalStore = getModalStore();

    const hostGameModal: ModalSettings = {
        type: 'component',
        component: { ref: EliminationGameForm }
    }

    const howToPlayModal: ModalSettings = {
        type: 'component',
        component: 'how-to-play-elimination'
    }

    const hostGame = () => {
        modalStore.trigger(hostGameModal);
    }

    const howToPlay = () => {
        modalStore.trigger(howToPlayModal);
    }
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <UserSidebar />
    </svelte:fragment>

    <svelte:fragment slot="main">
        <PageHeader title="ELIMINATION GAME" prevPage="/">
            <svelte:fragment slot="icon">
                <Swords size={28} />
            </svelte:fragment>
            <svelte:fragment slot="actions">
                <HelpButton ariaLabel="How to Play" on:click={howToPlay}>
                    <HelpCircle size={20} />
                </HelpButton>
                <ActionButton icon="plus" label="HOST GAME" on:click={hostGame} />
            </svelte:fragment>
        </PageHeader>
        <WaitingGames />
    </svelte:fragment>
</MainTemplate>
