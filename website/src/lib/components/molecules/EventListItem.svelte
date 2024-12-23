<script lang="ts">
	import BaseListItem from './BaseListItem.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import { deleteEvent, togglePinEvent } from '$lib/graphql/mutations/leaderboardAction.ts';
	import { getContextClient } from '@urql/svelte';
	import { userStore } from '$lib/stores/userStore';
	import { formatInTimeZone } from 'date-fns-tz';
	import { getModalStore, type ModalSettings } from '@skeletonlabs/skeleton';

	interface Props {
		leaderboardId: string;
		name: string;
		host: string;
		startTime: string;
		endTime: string;
		description?: string;
		isActive?: boolean;
		isPinned?: boolean;
		canDeleteEvent?: boolean;
		canPinEvent?: boolean;
		callback?: () => void;
	}

	let {
		leaderboardId,
		name,
		host,
		startTime,
		endTime,
		description,
		isPinned = false,
		isActive = false,
		canDeleteEvent = true,
		canPinEvent = true,
		callback
	}: Props = $props();

	const modalStore = getModalStore();
	const modal: ModalSettings = {
		type: 'confirm',
		title: 'Delete Event',
		body: 'Are you sure you want to delete this event?',
		response: (confirmed) => {
			if (confirmed) {
				deleteEventLeaderboard();
			}
		}
	};

	// TODO: use main chainId
	const client = getContextClient();

	const deleteEventLeaderboard = () => {
		deleteEvent(client, leaderboardId);
		callback?.();
	};

	const togglePin = () => {
		togglePinEvent(client, leaderboardId);
		callback?.();
	};

	// Get user's timezone
	const userTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;

	// Format function that converts timestamp to local time
	const formatLocalTime = (timestamp: string) => {
		try {
			return formatInTimeZone(
				new Date(Number(timestamp)),
				userTimeZone,
				"MMM d, yyyy 'at' h:mm a (zzz)"
			);
		} catch (error) {
			console.error(error);
			return 'Invalid timestamp';
		}
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
					<span class="me-1 font-semibold">Host:</span><span>{host}</span>
				</div>
				<div class="pb-1">
					<span class="me-1 font-semibold">Start:</span>
					<span>{formatLocalTime(startTime)}</span>
				</div>
				<div class="pb-1">
					<span class="me-1 font-semibold">End:</span>
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
		<div class="flex w-full flex-row items-end gap-2 md:flex-col">
			{#if $userStore?.isMod && canPinEvent}
				<div class="mt-2">
					<ActionButton label={isPinned ? 'Unpin' : 'Pin'} color="warning" onclick={togglePin} />
				</div>
			{/if}
			{#if (host === $userStore?.username || $userStore?.isMod) && canDeleteEvent}
				<div class="mt-2">
					<ActionButton label="Delete" color="warning" onclick={() => modalStore.trigger(modal)} />
				</div>
			{/if}
		</div>
	{/snippet}
</BaseListItem>
