<script lang="ts">
	import { goto } from '$app/navigation';
	import { getContextClient } from '@urql/svelte';
	import BaseListItem from './BaseListItem.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import TimeAgo from '../atoms/TimeAgo.svelte';
	import { joinGame } from '$lib/graphql/mutations';
	import type { EliminationGameDetails } from '$lib/types/eliminationGame';
	import { userStore } from '$lib/stores/userStore';

	let {
		gameId,
		gameName,
		playerCount,
		maxPlayers,
		host,
		createdTime,
		totalRounds,
		eliminatedPerTrigger,
		triggerIntervalSeconds,
		status
	}: EliminationGameDetails = $props();

	const client = getContextClient();
	let loading = $state(false);

	const handleJoinGame = (gameId: string) => {
		if (!$userStore.username) return;
		loading = true;
		joinGame(client, gameId);
	};

	const enterGame = (gameId: string) => {
		if (status === 'Waiting') {
			goto(`/elimination/${gameId}`);
		} else if (status === 'Active') {
			goto(`/game/${gameId}`);
		}
	};
</script>

<BaseListItem>
	{#snippet leftContent()}
		<div class="mb-1 flex items-center gap-2">
			<h3 class="text-lg font-semibold">{gameName}</h3>
			<span class="bg-surface-300-600-token rounded-full px-2 py-0.5 text-sm">
				{playerCount}/{maxPlayers} players
			</span>
		</div>
		<div class="flex items-center gap-4 text-sm text-surface-700">
			<div class="flex items-center gap-2">
				<div class="h-4 w-4 rounded bg-warning-500"></div>
				<span>{host}</span>
				<TimeAgo time={createdTime} />
			</div>
			<div class="flex items-center gap-2">
				<span>{totalRounds} rounds</span>
				<span>•</span>
				<span>{eliminatedPerTrigger} eliminated/trigger</span>
				<span>•</span>
				<span>{triggerIntervalSeconds}s interval</span>
			</div>
		</div>
	{/snippet}
	{#snippet rightContent()}
		{#if $userStore.username === host}
			<ActionButton label="Enter Game" color="important" onclick={() => enterGame(gameId)} />
		{:else if playerCount >= maxPlayers}
			<ActionButton label="Full" disabled={true} color="disabled" />
		{:else}
			<ActionButton
				label="Join Game"
				color="warning"
				{loading}
				onclick={() => handleJoinGame(gameId)}
			/>
		{/if}
	{/snippet}
</BaseListItem>
