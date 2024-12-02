<script lang="ts">
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

	const leaderboards = $derived(queryStore({ client, query: GET_LEADERBOARDS }));

	const activeEvents = $derived(
		$leaderboards?.data?.leaderboards.filter(
			(event: EventSettings) => Number(event.startTime) < Date.now()
		)
	);
	const upcomingEvents = $derived(
		$leaderboards?.data?.leaderboards.filter(
			(event: EventSettings) => Number(event.startTime) >= Date.now()
		)
	);
</script>

<div class="flex h-[80vh] w-full flex-col gap-6 overflow-y-auto pb-12 pt-6 md:h-[90vh]">
	{#if activeEvents?.length > 0}
		<div class="mx-auto flex max-w-4xl flex-col gap-4">
			<h2 class="text-xl font-bold text-yellow-600">Active Events</h2>
			{#each activeEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem isActive {...event} />
				{/if}
			{/each}
		</div>
	{/if}

	{#if upcomingEvents?.length > 0}
		<div class="mx-auto flex max-w-4xl flex-col gap-4">
			<h2 class="text-xl font-bold text-yellow-600">Upcoming Events</h2>
			{#each upcomingEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem {...event} />
				{/if}
			{/each}
		</div>
	{/if}
</div>
