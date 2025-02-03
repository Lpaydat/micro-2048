<script lang="ts">
	import { onMount } from 'svelte';
	import ArrowSwap from 'lucide-svelte/icons/arrow-left-right';
	import Star from 'lucide-svelte/icons/star';
	import type { LeaderboardSettings } from '$lib/graphql/mutations/leaderboardAction.ts';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import EventListItem from '../molecules/EventListItem.svelte';

	// TODO: use main chainId
	const client = getContextClient();

	const GET_LEADERBOARDS = gql`
		query GetLeaderboards {
			leaderboards {
				leaderboardId
				name
				description
				host
				startTime
				endTime
				isPinned
			}
		}
	`;

	let eventGroup = $state<'active' | 'upcoming' | 'past'>('active');

	const loopGroup = () => {
		eventGroup =
			eventGroup === 'active' ? 'upcoming' : eventGroup === 'upcoming' ? 'past' : 'active';
	};

	const leaderboards = $derived(queryStore({ client, query: GET_LEADERBOARDS }));
	const sortedEvents = $derived(
		$leaderboards?.data?.leaderboards.sort((a: LeaderboardSettings, b: LeaderboardSettings) => {
			// First sort by start time
			const startDiff = Number(a.startTime) - Number(b.startTime);
			// If start times are equal, sort by end time
			return startDiff !== 0 ? startDiff : Number(a.endTime) - Number(b.endTime);
		})
	);

	const pinnedEvents = $derived(
		sortedEvents?.filter(
			(event: LeaderboardSettings) => event.isPinned && Number(event.endTime) >= Date.now()
		)
	);

	const activeEvents = $derived(
		sortedEvents?.filter((event: LeaderboardSettings) => {
			const now = Date.now();
			return now >= Number(event.startTime) && now < Number(event.endTime);
		})
	);

	const upcomingEvents = $derived(
		sortedEvents?.filter((event: LeaderboardSettings) => Number(event.startTime) >= Date.now())
	);

	const pastEvents = $derived(
		sortedEvents?.filter((event: LeaderboardSettings) => Number(event.endTime) < Date.now())
	);

	const callback = () => {
		setTimeout(() => {
			leaderboards.reexecute({ requestPolicy: 'network-only' });
		}, 500);
	};

	let titleClass = $derived(pinnedEvents?.length > 0 ? 'text-lg mt-4' : 'text-xl');

	let interval: NodeJS.Timeout;
	onMount(() => {
		leaderboards.reexecute({ requestPolicy: 'network-only' });

		interval = setInterval(() => {
			leaderboards.reexecute({ requestPolicy: 'network-only' });
		}, 5000);

		return () => clearInterval(interval);
	});
</script>

<div
	class="flex h-[calc(100vh-136px)] w-full flex-col gap-6 overflow-y-auto pb-6 pt-6 md:h-[calc(100vh-84px)]"
>
	<div class="mx-auto flex w-full max-w-3xl flex-col gap-4">
		{#if pinnedEvents?.length > 0}
			<div class="border-s-8 border-red-600">
				<h2
					class="card flex w-fit flex-row items-center gap-2 rounded-none bg-black/30 px-4 py-2 text-xl font-extrabold text-red-400 shadow-lg"
				>
					<Star size={20} fill="#F87171" strokeWidth={0} />
					Events
				</h2>
			</div>
			{#each pinnedEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem {...event} {callback} isPinned />
				{/if}
			{/each}
		{/if}

		<button
			type="button"
			onclick={loopGroup}
			class="ms-3 flex flex-row items-center gap-2 py-1 {titleClass} font-bold text-yellow-500 md:ms-0 md:py-2"
		>
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
					<EventListItem
						isActive
						{...event}
						{callback}
						className={pinnedEvents?.length ? 'opacity-50 transition-opacity duration-300' : ''}
					/>
				{/if}
			{/each}
		{:else if eventGroup === 'upcoming' && upcomingEvents?.length > 0}
			{#each upcomingEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem
						{...event}
						{callback}
						className={pinnedEvents?.length ? 'opacity-50 transition-opacity duration-300' : ''}
					/>
				{/if}
			{/each}
		{:else if eventGroup === 'past' && pastEvents?.length > 0}
			{#each pastEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem
						canDeleteEvent={false}
						canPinEvent={false}
						{...event}
						{callback}
						className={pinnedEvents?.length ? 'opacity-50 transition-opacity duration-300' : ''}
					/>
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
