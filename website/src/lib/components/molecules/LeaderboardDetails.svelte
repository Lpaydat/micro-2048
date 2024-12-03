<script lang="ts">
	import { onDestroy } from 'svelte';

	interface Props {
		leaderboardId?: string;
		showName?: boolean;
		name?: string;
		host?: string;
		totalBoards?: number;
		totalPlayers?: number;
		endTime?: string;
	}

	let {
		host,
		totalBoards,
		totalPlayers,
		name,
		endTime,
		showName = false,
		leaderboardId
	}: Props = $props();

	let remainingTime = $state('');
	let timer: NodeJS.Timeout;
	let isTimeSet = $state(false);

	const calculateRemainingTime = () => {
		console.log('name', name);
		const end = Number(endTime);
		const now = Date.now();
		const diff = end - now;

		if (diff > 0) {
			const months = Math.floor(diff / (1000 * 60 * 60 * 24 * 30));
			const days = Math.floor((diff % (1000 * 60 * 60 * 24 * 30)) / (1000 * 60 * 60 * 24));
			const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
			const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
			const seconds = Math.floor((diff % (1000 * 60)) / 1000);

			if (months > 0) {
				remainingTime = `${months} mo ${days} d`;
			} else if (days > 0) {
				remainingTime = `${days} d ${hours} hr`;
			} else if (hours > 0) {
				remainingTime = `${hours} hr ${minutes} min`;
			} else if (minutes > 0) {
				remainingTime = `${minutes} min ${seconds} sec`;
			} else {
				remainingTime = `${seconds} sec`;
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
		} else {
			remainingTime = 'Event has ended';
		}
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
	{#if endTime && remainingTime}
		<div class="flex items-center gap-2 md:text-sm">
			<span class="font-semibold text-surface-400">Time Left:</span>
			<span class="game-details font-bold text-green-600">{remainingTime}</span>
		</div>
	{/if}
</div>

<style>
	.game-details {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
	}
	.text-green-600 {
		color: #16a34a; /* Tailwind CSS green-600 */
	}
</style>
