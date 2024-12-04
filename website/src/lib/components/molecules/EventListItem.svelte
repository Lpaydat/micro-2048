<script lang="ts">
	import BaseListItem from './BaseListItem.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import { deleteEvent } from '$lib/graphql/mutations/createEvent';
	import { getContextClient } from '@urql/svelte';
	import { userStore } from '$lib/stores/userStore';
	import { formatInTimeZone } from 'date-fns-tz';

	interface Props {
		leaderboardId: string;
		name: string;
		host: string;
		startTime: string;
		endTime: string;
		description?: string;
		isActive?: boolean;
		canDeleteEvent?: boolean;
	}

	let {
		leaderboardId,
		name,
		host,
		startTime,
		endTime,
		description,
		isActive = false,
		canDeleteEvent = true
	}: Props = $props();

	const client = getContextClient();

	const deleteEventLeaderboard = (event: MouseEvent) => {
		deleteEvent(client, leaderboardId);
	};

	// Get user's timezone
	const userTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;

	// Format function that converts timestamp to local time
	const formatLocalTime = (timestamp: string) => {
		return formatInTimeZone(
			new Date(Number(timestamp)),
			userTimeZone,
			"MMM d, yyyy 'at' h:mm a (zzz)"
		);
	};
</script>

<BaseListItem>
	{#snippet leftContent()}
		<a href={`/events/${leaderboardId}`}>
			<div class="flex w-full items-center justify-between pb-3">
				<h3 class="text-xl font-bold text-gray-800">{name}</h3>
			</div>
			<div class="text-sm text-gray-600">
				<div class="pb-1">
					<span class="me-2 font-semibold">Host:</span><span>{host}</span>
				</div>
				<div class="pb-1">
					<span class="me-2 font-semibold">Start:</span>
					<span>{formatLocalTime(startTime)}</span>
				</div>
				<div class="pb-1">
					<span class="me-2 font-semibold">End:</span>
					<span>{formatLocalTime(endTime)}</span>
				</div>
				{#if description}
					<div class="mt-3 border-t-2 border-surface-200 pt-4 text-gray-700">
						{description}
					</div>
				{/if}
			</div>
		</a>
	{/snippet}
	{#snippet rightContent()}
		{#if host === $userStore?.username && canDeleteEvent}
			<div class="mt-4 flex justify-end">
				<ActionButton label="Delete" color="warning" onclick={deleteEventLeaderboard} />
			</div>
		{/if}
	{/snippet}
</BaseListItem>
