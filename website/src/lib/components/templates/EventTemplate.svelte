<script lang="ts">
	import Calendar from 'lucide-svelte/icons/calendar-1';
	import MainTemplate from '$lib/components/organisms/MainTemplate.svelte';
	import UserSidebar from '$lib/components/organisms/UserSidebar.svelte';
	import ActionButton from '$lib/components/atoms/ActionButton.svelte';
	import PageHeader from '$lib/components/molecules/PageHeader.svelte';
	import EventLeaderboardForm from '$lib/components/organisms/EventLeaderboardForm.svelte';
	import { getModalStore, type ModalSettings, type ModalStore } from '@skeletonlabs/skeleton';
	import EventList from '$lib/components/organisms/EventList.svelte';
	import { userStore } from '$lib/stores/userStore';

	const modalStore: ModalStore = getModalStore();

	const createEventModal: ModalSettings = {
		type: 'component',
		component: { ref: EventLeaderboardForm }
	};

	const createEvent = () => {
		modalStore.trigger(createEventModal);
	};
</script>

<MainTemplate>
	{#snippet sidebar()}
		<UserSidebar />
	{/snippet}

	{#snippet main()}
		<PageHeader color="cyan" title="EVENTS" prevPage="/">
			{#snippet icon()}
				<Calendar size={28} />
			{/snippet}
			{#snippet actions()}
				{#if $userStore.username}
					<ActionButton label="CREATE EVENT" onclick={createEvent} />
				{/if}
			{/snippet}
		</PageHeader>
		<EventList />
	{/snippet}
</MainTemplate>
