<script lang="ts">
	import BaseListItem from './BaseListItem.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';

	interface Props {
		leaderboardId: string;
		name: string;
		host: string;
		startTime: string;
		endTime: string;
		description: string;
		isActive?: boolean;
	}

	let {
		leaderboardId,
		name = 'Hello World!',
		host,
		startTime,
		endTime,
		description = 'This is a mock description. It is intended to provide a placeholder for the actual event description. The real description will be more detailed and informative, giving users a clear understanding of the event.',
		isActive = false
	}: Props = $props();

	const formatTimeUTC = (time: string) => {
		return new Date(Number(time)).toLocaleString('en-US', {
			timeZone: 'UTC',
			year: 'numeric',
			month: 'long',
			day: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
			hour12: true
		});
	};
</script>

<BaseListItem>
	{#snippet leftContent()}
		<div class="flex justify-between items-center mb-2">
			<h3 class="text-xl font-bold text-gray-800">{name || 'Hello World!'}</h3>
		</div>
		<div class="text-sm text-gray-600">
			<div class="mb-1">
				<span class="font-semibold me-2">Host:</span><span>{host}</span>
			</div>
			<div class="mb-1">
				<span class="font-semibold me-2">Start:</span><span>{formatTimeUTC(startTime)} UTC</span>
			</div>
			<div class="mb-1">
				<span class="font-semibold me-2">End:</span><span>{formatTimeUTC(endTime)} UTC</span>
			</div>
			{#if description}
				<div class="mt-3 text-gray-700 pt-4 border-surface-200 border-t-2">
					{description}
				</div>
			{/if}
		</div>
	{/snippet}
	{#snippet rightContent()}
		<div class="flex justify-end mt-4">
			{#if isActive}
				<a href={`/events/${leaderboardId}`}>
					<ActionButton label="Leaderboard" color="warning" />
				</a>
			{/if}
		</div>
	{/snippet}
</BaseListItem>
