<script lang="ts">
	import { onMount } from 'svelte';
    import ArrowSwap from 'lucide-svelte/icons/arrow-left-right';
	import type { EventSettings } from '$lib/graphql/mutations/createEvent';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import EventListItem from '../molecules/EventListItem.svelte';

	const client = getContextClient();

	const GET_LEADERBOARDS = gql`
		query GetLeaderboards {
			leaderboards {
				leaderboardId
				name
				host
				startTime
				endTime
			}
		}
	`;

	let eventGroup = $state<'active' | 'upcoming' | 'past'>('active');

	const loopGroup = () => {
		eventGroup =
			eventGroup === 'active' ? 'upcoming' : eventGroup === 'upcoming' ? 'past' : 'active';
	};

	const leaderboards = $derived(queryStore({ client, query: GET_LEADERBOARDS }));

	const activeEvents = $derived(
		$leaderboards?.data?.leaderboards.filter((event: EventSettings) => {
			const now = Date.now();
			return now >= Number(event.startTime) && now < Number(event.endTime);
		})
	);
	const upcomingEvents = $derived(
		$leaderboards?.data?.leaderboards.filter(
			(event: EventSettings) => Number(event.startTime) >= Date.now()
		)
	);
	const pastEvents = $derived(
		$leaderboards?.data?.leaderboards.filter(
			(event: EventSettings) => Number(event.endTime) < Date.now()
		)
	);

	let interval: NodeJS.Timeout;
	onMount(() => {
		leaderboards.reexecute({ requestPolicy: 'network-only' });

		interval = setInterval(() => {
			leaderboards.reexecute({ requestPolicy: 'network-only' });
		}, 5000);

		return () => clearInterval(interval);
	});
</script>

<div class="flex h-[80vh] w-full flex-col gap-6 overflow-y-auto pb-12 pt-6 md:h-[90vh]">
	<div class="mx-auto flex w-full max-w-3xl flex-col gap-4">
		<button type="button" onclick={loopGroup} class="flex items-center flex-row gap-2 ms-3 md:ms-0 py-1 md:py-2 text-xl font-bold text-yellow-600">
			{eventGroup === 'active'
				? 'Active Events'
				: eventGroup === 'upcoming'
					? 'Upcoming Events'
					: 'Past Events'}
			<ArrowSwap size={20} />
		</button>

		{#if eventGroup === 'active' && activeEvents?.length > 0}
			{#each activeEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem isActive {...event} />
				{/if}
			{/each}
		{:else if eventGroup === 'upcoming' && upcomingEvents?.length > 0}
			{#each upcomingEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem {...event} />
				{/if}
			{/each}
		{:else if eventGroup === 'past' && pastEvents?.length > 0}
			{#each pastEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem canDeleteEvent={false} {...event} />
				{/if}
			{/each}
		{:else}
			<div class="flex h-[60vh] w-full items-center justify-center">
				<h2 class="text-md font-bold text-gray-400">
					No {eventGroup} events
				</h2>
			</div>
		{/if}
	</div>
</div>
