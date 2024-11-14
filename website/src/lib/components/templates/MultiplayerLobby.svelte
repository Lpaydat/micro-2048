<script lang="ts">
    import { getModalStore } from '@skeletonlabs/skeleton';
    import Swords from 'lucide-svelte/icons/swords';

	import ActionButton from '../atoms/ActionButton.svelte';
	import PageHeader from "../molecules/PageHeader.svelte";
    import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import WaitingGames from "../organisms/WaitingGames.svelte";
    import EliminationGameForm from '../organisms/EliminationGameForm.svelte';
	import type { ModalSettings, ModalStore } from '@skeletonlabs/skeleton';

    const modal: ModalSettings = {
        type: 'component',
        component: { ref: EliminationGameForm }
    }

    const modalStore: ModalStore = getModalStore();

    const hostGame = () => {
        modalStore.trigger(modal);
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
                <ActionButton icon="plus" label="HOST GAME" on:click={hostGame} />
            </svelte:fragment>
        </PageHeader>
        <WaitingGames />
    </svelte:fragment>
</MainTemplate>
