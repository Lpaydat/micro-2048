<script lang="ts">
	import BaseListItem from './BaseListItem.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import { formatTimeUTC } from '$lib/utils/formatTimeUTC';

	interface Props {
		leaderboardId: string;
		name: string;
		host: string;
		startTime: string;
		endTime: string;
		description?: string;
		isActive?: boolean;
	}

	let {
		leaderboardId,
		name,
		host,
		startTime,
		endTime,
		description,
		isActive = false
	}: Props = $props();
</script>

<BaseListItem>
	{#snippet leftContent()}
		<div class="mb-2 flex items-center justify-between">
			<h3 class="text-xl font-bold text-gray-800">{name}</h3>
		</div>
		<div class="text-sm text-gray-600">
			<div class="mb-1">
				<span class="me-2 font-semibold">Host:</span><span>{host}</span>
			</div>
			<div class="mb-1">
				<span class="me-2 font-semibold">Start:</span><span>{formatTimeUTC(startTime)} UTC</span>
			</div>
			<div class="mb-1">
				<span class="me-2 font-semibold">End:</span><span>{formatTimeUTC(endTime)} UTC</span>
			</div>
			{#if description}
				<div class="mt-3 border-t-2 border-surface-200 pt-4 text-gray-700">
					{description}
				</div>
			{/if}
		</div>
	{/snippet}
	{#snippet rightContent()}
		<div class="mt-4 flex justify-end">
			{#if isActive}
				<a href={`/events/${leaderboardId}`}>
					<ActionButton label="Leaderboard" color="warning" />
				</a>
			{/if}
		</div>
	{/snippet}
</BaseListItem>
