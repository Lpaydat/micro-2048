<script lang="ts">
	import { onDestroy } from 'svelte';

	interface Props {
		leaderboardId?: string;
		showName?: boolean;
		name?: string;
		host?: string;
		totalBoards?: number;
		totalPlayers?: number;
		startTime?: string;
		endTime?: string;
	}

	let {
		host,
		totalBoards,
		totalPlayers,
		name,
		endTime,
		startTime,
		showName = false,
		leaderboardId
	}: Props = $props();

	let remainingTime = $state('');
	let label = $state('');
	let timer: NodeJS.Timeout;
	let isTimeSet = $state(false);

	const calculateRemainingTime = () => {
		const start = Number(startTime);
		const end = Number(endTime);
		const now = Date.now();

		if (start > now) {
			// Event hasn't started yet
			const diff = start - now;
			[label, remainingTime] = formatTime(diff, 'Starts In');
		} else if (end > now) {
			// Event is ongoing
			const diff = end - now;
			[label, remainingTime] = formatTime(diff, 'Time Left');
		} else {
			// Event has ended
			label = 'Event has ended';
		}
	};

	const formatTime = (diff: number, label: string): [string, string] => {
		const months = Math.floor(diff / (1000 * 60 * 60 * 24 * 30));
		const days = Math.floor((diff % (1000 * 60 * 60 * 24 * 30)) / (1000 * 60 * 60 * 24));
		const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
		const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
		const seconds = Math.floor((diff % (1000 * 60)) / 1000);

		let timeString = '';
		if (months > 0) {
			timeString = `${months} mo ${days} d`;
		} else if (days > 0) {
			timeString = `${days} d ${hours} hr`;
		} else if (hours > 0) {
			timeString = `${hours} hr ${minutes} min`;
		} else if (minutes > 0) {
			timeString = `${minutes} min ${seconds} sec`;
		} else {
			timeString = `${seconds} sec`;
		}

		// Set dynamic interval
		let interval;
		if (months > 0 || days > 0) {
			interval = 3600000; // Update every hour
		} else if (hours > 0) {
			interval = 60000; // Update every minute
		} else {
			interval = 1000; // Update every second
		}

		clearInterval(timer);
		timer = setInterval(calculateRemainingTime, interval);

		return [label, timeString];
	};

	$effect(() => {
		if (!isTimeSet && endTime) {
			isTimeSet = true;
			calculateRemainingTime();
		}
	});

	onDestroy(() => {
		clearInterval(timer);
	});
</script>

<div class="game-details relative text-xs md:text-base">
	{#if name && showName}
		<div class="flex items-center gap-2 text-lg">
			<a href={`/events/${leaderboardId}`}>
				<span class="game-details line-clamp-4 font-bold text-orange-600 hover:underline">
					{name}
				</span>
			</a>
		</div>
	{/if}
	<div class="flex items-center justify-between gap-2 md:gap-4">
		<div class="flex items-center gap-2 text-surface-200 md:gap-3">
			[<span class="font-bold text-cyan-600">{host}</span>]
		</div>
		<div class="flex items-center gap-2">
			<span class="font-semibold text-surface-400">Boards:</span>
			<span class="game-details font-bold text-orange-600">{totalBoards}</span>
		</div>
		<div class="flex items-center gap-2">
			<span class="font-semibold text-surface-400">Players:</span>
			<span class="game-details font-bold text-orange-600">{totalPlayers}</span>
		</div>
	</div>
	{#if (startTime || endTime) && remainingTime}
		<div class="flex items-center gap-2 md:text-sm">
			<span class="font-semibold text-surface-400">{label}:</span>
			{#if remainingTime}
				<span class="game-details font-bold text-green-600">{remainingTime}</span>
			{/if}
		</div>
	{/if}
</div>

<style>
	.game-details {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
	}
</style>
