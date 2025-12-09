<script lang="ts">
	import { onMount } from 'svelte';

	import Star from 'lucide-svelte/icons/star';
	import type { LeaderboardState } from '$lib/graphql/mutations/leaderboardAction.ts';
	import { gql, queryStore } from '@urql/svelte';
	import EventListItem from '../molecules/EventListItem.svelte';
	import { getClient } from '$lib/client';
	import { chainId as mainChainId } from '$lib/constants';
	import { userStore } from '$lib/stores/userStore';
	import { requestLeaderboardRefresh } from '$lib/graphql/mutations/requestLeaderboardRefresh';

	const client = getClient(mainChainId, true);
	
	// Track leaderboards we've already initialized to avoid duplicate calls
	let initializedLeaderboards = new Set<string>();
	const GET_LEADERBOARDS = gql`
		query GetLeaderboards($filter: TournamentFilter) {
			leaderboards(filter: $filter) {
				leaderboardId
				chainId
				name
				description
				host
				startTime
				endTime
				isPinned
				totalBoards
				totalPlayers
				shardIds
			}
		}
	`;

	let eventGroup = $state<'active' | 'upcoming' | 'past'>('active');

	const loopGroup = () => {
		eventGroup =
			eventGroup === 'active' ? 'upcoming' : eventGroup === 'upcoming' ? 'past' : 'active';
	};

	const leaderboards = $derived(
		queryStore({
			client,
			query: GET_LEADERBOARDS,
			variables: { filter: 'ALL' }
		})
	);

	const sortedEvents = $derived(
		$leaderboards?.data?.leaderboards
			.filter((event: LeaderboardState) => {
				// Filter out only the default main chain leaderboard and truly invalid entries
				// Allow tournaments with startTime/endTime of 0 (unlimited tournaments)
				return event.name && event.leaderboardId;
			})
			.sort((a: LeaderboardState, b: LeaderboardState) => {
				// First sort by start time
				const startDiff = Number(a.startTime) - Number(b.startTime);
				// If start times are equal, sort by end time
				return startDiff !== 0 ? startDiff : Number(a.endTime) - Number(b.endTime);
			})
	);

	const pinnedEvents = $derived(
		sortedEvents?.filter((event: LeaderboardState) => {
			if (!event.isPinned) return false;

			const endTime = Number(event.endTime);
			// Unlimited tournaments (endTime = 0) are always pinned if marked as such
			if (endTime === 0) return true;

			// Regular tournaments: only show if not ended yet
			return endTime >= Date.now();
		})
	);

	const activeEvents = $derived(
		sortedEvents?.filter((event: LeaderboardState) => {
			const now = Date.now();
			const startTime = Number(event.startTime);
			const endTime = Number(event.endTime);

			// Unlimited tournaments (timestamp = 0) are always active
			if (startTime === 0 && endTime === 0) return true;

			// Regular tournaments: check time bounds
			return now >= startTime && now < endTime;
		})
	);

	const upcomingEvents = $derived(
		sortedEvents?.filter((event: LeaderboardState) => {
			const startTime = Number(event.startTime);
			const endTime = Number(event.endTime);

			// Unlimited tournaments are not "upcoming" (they're always active)
			if (startTime === 0 && endTime === 0) return false;

			// Regular tournaments: start time in future
			return startTime >= Date.now();
		})
	);

	const pastEvents = $derived(
		sortedEvents?.filter((event: LeaderboardState) => {
			const endTime = Number(event.endTime);

			// Unlimited tournaments are never "past" (they're always active)
			if (endTime === 0) return false;

			// Regular tournaments: end time in past
			return endTime < Date.now();
		})
	);

	// Sorted versions for better UX
	const sortedActiveEvents = $derived(
		activeEvents?.sort((a: LeaderboardState, b: LeaderboardState) => {
			const aIsUnlimited = Number(a.endTime) === 0;
			const bIsUnlimited = Number(b.endTime) === 0;

			// Put regular tournaments before unlimited ones (time-sensitive first)
			if (!aIsUnlimited && bIsUnlimited) return -1;
			if (aIsUnlimited && !bIsUnlimited) return 1;

			// Both are the same type, sort within their group
			if (!aIsUnlimited && !bIsUnlimited) {
				// Regular tournaments: end time ascending (ending soon first)
				const endDiff = Number(a.endTime) - Number(b.endTime);
				if (endDiff !== 0) return endDiff;
			}

			// Both unlimited or within unlimited group: sort by popularity
			const playerDiff = b.totalPlayers - a.totalPlayers;
			if (playerDiff !== 0) return playerDiff;

			// Final fallback: start time descending (longer running first)
			return Number(b.startTime) - Number(a.startTime);
		})
	);

	const sortedUpcomingEvents = $derived(
		upcomingEvents?.sort((a: LeaderboardState, b: LeaderboardState) => {
			// Primary: Start time ascending (starting soon first)
			const startDiff = Number(a.startTime) - Number(b.startTime);
			if (startDiff !== 0) return startDiff;

			// Secondary: Total players descending (most anticipated first)
			return b.totalPlayers - a.totalPlayers;
		})
	);

	const sortedPastEvents = $derived(
		pastEvents?.sort((a: LeaderboardState, b: LeaderboardState) => {
			// Primary: End time descending (most recent first)
			const endDiff = Number(b.endTime) - Number(a.endTime);
			if (endDiff !== 0) return endDiff;

			// Secondary: Total players descending (most participated first)
			return b.totalPlayers - a.totalPlayers;
		})
	);

	const callback = () => {
		setTimeout(() => {
			leaderboards.reexecute({ requestPolicy: 'network-only' });
		}, 500);
	};

	let titleClass = $derived(pinnedEvents?.length > 0 ? 'text-lg mt-4' : 'text-xl');

	// 🚀 Auto-initialize new leaderboards created by the current user
	// This sends the first transaction to the leaderboard chain so config message is processed
	$effect(() => {
		const currentUser = $userStore.username;
		const events = $leaderboards?.data?.leaderboards;
		
		if (!currentUser || !events) return;
		
		for (const event of events) {
			// Check if this is a new leaderboard created by the current user that we haven't initialized
			if (
				event.leaderboardId &&
				event.host === currentUser &&
				!initializedLeaderboards.has(event.leaderboardId)
			) {
				// Mark as initialized immediately to prevent duplicate calls
				initializedLeaderboards.add(event.leaderboardId);
				
				// Call updateLeaderboard to initialize the chain
				try {
					const leaderboardClient = getClient(event.leaderboardId, true);
					const result = requestLeaderboardRefresh(leaderboardClient);
					if (result) {
						result.subscribe((res) => {
							if (res.error) {
								console.warn(`Leaderboard ${event.leaderboardId} initialization failed:`, res.error);
							} else {
								console.log(`🚀 Leaderboard ${event.leaderboardId} initialized`);
							}
						});
					}
				} catch (error) {
					console.warn(`Failed to initialize leaderboard ${event.leaderboardId}:`, error);
				}
			}
		}
	});

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

		<div class="ms-3 flex flex-col gap-1 md:ms-0">
			<button
				type="button"
				onclick={loopGroup}
				class="flex flex-row items-center gap-2 py-1 {titleClass} cursor-pointer font-bold text-yellow-500 transition-colors hover:text-yellow-400 md:py-2"
			>
				{eventGroup === 'active'
					? 'Active Events'
					: eventGroup === 'upcoming'
						? 'Upcoming Events'
						: 'Completed Events'}
			</button>
			<div class="flex items-center gap-1 text-xs text-gray-400">
				<span>Click to toggle:</span>
				<span class={eventGroup === 'active' ? 'font-semibold text-yellow-500' : 'text-gray-500'}
					>Active</span
				>
				<span class="text-gray-300">•</span>
				<span class={eventGroup === 'upcoming' ? 'font-semibold text-yellow-500' : 'text-gray-500'}
					>Upcoming</span
				>
				<span class="text-gray-300">•</span>
				<span class={eventGroup === 'past' ? 'font-semibold text-yellow-500' : 'text-gray-500'}
					>Completed</span
				>
			</div>
		</div>

		{#if eventGroup === 'active' && sortedActiveEvents?.length > 0}
			{#each sortedActiveEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem
						isActive
						{...event}
						{callback}
						className={pinnedEvents?.length ? 'opacity-50 transition-opacity duration-300' : ''}
					/>
				{/if}
			{/each}
		{:else if eventGroup === 'upcoming' && sortedUpcomingEvents?.length > 0}
			{#each sortedUpcomingEvents as event}
				{#if event && event.leaderboardId}
					<EventListItem
						{...event}
						{callback}
						className={pinnedEvents?.length ? 'opacity-50 transition-opacity duration-300' : ''}
					/>
				{/if}
			{/each}
		{:else if eventGroup === 'past' && sortedPastEvents?.length > 0}
			{#each sortedPastEvents as event}
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
