<script lang="ts">
	import BaseListItem from './BaseListItem.svelte';
	import ActionButton from '../atoms/ActionButton.svelte';
	import {
		deleteLeaderboard,
		togglePinLeaderboard
	} from '$lib/graphql/mutations/leaderboardAction.ts';
	import { getContextClient } from '@urql/svelte';
	import { userStore } from '$lib/stores/userStore';
	import { formatInTimeZone } from 'date-fns-tz';
	import { getModalStore, type ModalSettings } from '@skeletonlabs/skeleton';
	import { onDestroy } from 'svelte';
	import Music from 'lucide-svelte/icons/music';
	import { RhythmEngine } from '$lib/game/rhythmEngine.js';

	interface Props {
		leaderboardId: string;
		name: string;
		className?: string;
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
		className,
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

	// Check if this is a rhythm mode tournament
	const isRhythmMode = $derived(RhythmEngine.isRhythmMode(description));
	
	// Extract rhythm settings for display
	const rhythmSettings = $derived(() => RhythmEngine.getDisplayInfo(description));

	// Clean description (remove rhythm settings tag)
	const cleanDescription = $derived(RhythmEngine.cleanDescription(description));

	// Add reactive timestamp and interval for active state updates
	let now = $state(Date.now());
	const intervalId = setInterval(() => (now = Date.now()), 1000);
	onDestroy(() => clearInterval(intervalId));

	const isCurrentlyActive = $derived(() => {
		const start = Number(startTime);
		const end = Number(endTime);

		// Unlimited tournaments (both timestamps = 0) are always active
		if (start === 0 && end === 0) return true;

		// Regular tournaments: check time bounds
		return now >= start && now < end;
	});
	const isPinnedAndActive = $derived(isPinned && isCurrentlyActive());
	
	// Rhythm mode styling - purple/violet gradient theme with pulse animation on hover
	const rhythmClass = $derived(
		isRhythmMode
			? 'rhythm-item !bg-gradient-to-br !from-violet-100 !to-purple-100 !border-2 !border-violet-400'
			: ''
	);
	
	const activeClass = $derived(
		isPinned && isCurrentlyActive()
			? isRhythmMode
				? '!bg-gradient-to-br !from-violet-200 !to-purple-200 !border-2 !border-violet-600 !ring-1 !ring-violet-600 shadow-[0_0_15px_rgba(139,92,246,0.5)]'
				: '!bg-orange-200 !border-2 !border-orange-800 !ring-1 !ring-orange-800 shadow-[0_0_15px_rgba(234,88,12,0.5)]'
			: ''
	);
	const itemClass = $derived(`${className} ${rhythmClass} ${activeClass}`);

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
		deleteLeaderboard(client, leaderboardId);
		callback?.();
	};

	const togglePin = () => {
		togglePinLeaderboard(client, leaderboardId);
		callback?.();
	};

	// Get user's timezone
	const userTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;

	// Format function that converts timestamp to local time
	const formatLocalTime = (timestamp: string) => {
		try {
			const numTimestamp = Number(timestamp);
			if (numTimestamp === 0) {
				return 'Unlimited';
			}
			if (!numTimestamp || !isFinite(numTimestamp) || numTimestamp < 0) {
				return 'Invalid timestamp';
			}
			const date = new Date(numTimestamp);
			if (!isFinite(date.getTime())) {
				return 'Invalid timestamp';
			}
			return formatInTimeZone(date, userTimeZone, "MMM d, yyyy 'at' h:mm a (zzz)");
		} catch (error) {
			console.error('Date formatting error:', error);
			return 'Invalid timestamp';
		}
	};
</script>

<BaseListItem className={itemClass}>
	{#snippet leftContent()}
		<a href={`/events/${leaderboardId}`} class={isRhythmMode ? 'rhythm-link' : ''}>
			<div class="flex w-full items-center justify-between pb-3">
				<div class="flex flex-wrap items-center gap-2">
					<h3 class="text-xl font-bold {isRhythmMode ? 'text-violet-800' : 'text-gray-800'}">{name}</h3>
					{#if isRhythmMode}
						<span class="inline-flex items-center gap-1 rounded-full bg-violet-600 px-2.5 py-0.5 text-xs font-medium text-white">
							<Music size={12} />
							Rhythm
						</span>
					{/if}
					{#if isPinnedAndActive}
						<span class="badge-primary badge gap-2 px-2 py-1 text-sm"> 📌 Pinned & Active </span>
					{/if}
				</div>
			</div>
			<div class="text-sm {isRhythmMode ? 'text-violet-700' : 'text-gray-600'}">
				<div class="pb-1">
					<span class="me-1 font-semibold">Creator:</span><span>{host}</span>
				</div>
				<div class="pb-1">
					<span class="me-1 font-semibold">Start:</span>
					<span>{formatLocalTime(startTime)}</span>
				</div>
				<div class="pb-1">
					<span class="me-1 font-semibold">End:</span>
					<span>{formatLocalTime(endTime)}</span>
				</div>
				{#if isRhythmMode && rhythmSettings()}
					{@const settings = rhythmSettings()}
					<div class="mt-2 flex flex-wrap gap-2">
						{#if settings?.bpm && settings.bpm > 0}
							<span class="inline-flex items-center rounded-md bg-violet-100 px-2 py-1 text-xs font-medium text-violet-700 ring-1 ring-inset ring-violet-300">
								{settings.bpm} BPM
							</span>
						{/if}
						<span class="inline-flex items-center rounded-md bg-violet-100 px-2 py-1 text-xs font-medium text-violet-700 ring-1 ring-inset ring-violet-300">
							±{settings?.tolerance}ms
						</span>
						{#if settings?.useMusic}
							<span class="inline-flex items-center gap-1 rounded-md bg-violet-100 px-2 py-1 text-xs font-medium text-violet-700 ring-1 ring-inset ring-violet-300">
								<Music size={10} />
								{#if settings.trackName}
									{settings.trackName}
								{:else}
									Random Track
								{/if}
							</span>
						{:else}
							<span class="inline-flex items-center gap-1 rounded-md bg-violet-100 px-2 py-1 text-xs font-medium text-violet-700 ring-1 ring-inset ring-violet-300">
								🔊 Metronome
							</span>
						{/if}
					</div>
				{/if}
				{#if cleanDescription}
					<div class="mt-3 border-t-2 {isRhythmMode ? 'border-violet-200' : 'border-surface-200'} pt-4 {isRhythmMode ? 'text-violet-800' : 'text-gray-700'}">
						{cleanDescription}
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

<style>
	/* Rhythm mode hover effects - 65 BPM (923ms per beat) */
	/* Single smooth pulse then rest */
	:global(.rhythm-item) {
		position: relative;
		overflow: hidden;
	}

	:global(.rhythm-item:hover) {
		border-color: rgb(139, 92, 246) !important;
		animation: pulse-beat 923ms ease-out infinite;
	}

	/* Single beat: smooth rise, smooth fall, then rest
	   0%: rest
	   20%: peak
	   45%: back to rest
	   100%: rest continues */
	@keyframes pulse-beat {
		0% {
			transform: scale(1);
			box-shadow: 
				0 0 8px rgba(139, 92, 246, 0.15),
				0 0 16px rgba(139, 92, 246, 0.08);
		}
		20% {
			transform: scale(1.012);
			box-shadow: 
				0 0 18px rgba(139, 92, 246, 0.45),
				0 0 35px rgba(139, 92, 246, 0.2);
		}
		45% {
			transform: scale(1);
			box-shadow: 
				0 0 8px rgba(139, 92, 246, 0.15),
				0 0 16px rgba(139, 92, 246, 0.08);
		}
		100% {
			transform: scale(1);
			box-shadow: 
				0 0 8px rgba(139, 92, 246, 0.15),
				0 0 16px rgba(139, 92, 246, 0.08);
		}
	}

	/* Music icon pulse */
	:global(.rhythm-item:hover [data-lucide="music"]) {
		animation: music-pulse 923ms ease-out infinite;
	}

	@keyframes music-pulse {
		0% { transform: scale(1); }
		20% { transform: scale(1.15); }
		45% { transform: scale(1); }
		100% { transform: scale(1); }
	}

	/* Rhythm badge pulse */
	:global(.rhythm-item:hover .badge) {
		animation: badge-pulse 923ms ease-out infinite;
	}

	@keyframes badge-pulse {
		0% { transform: scale(1); }
		20% { transform: scale(1.05); }
		45% { transform: scale(1); }
		100% { transform: scale(1); }
	}

	/* BPM/tolerance tags glow pulse */
	:global(.rhythm-item:hover .ring-violet-300) {
		animation: tag-pulse 923ms ease-out infinite;
	}

	@keyframes tag-pulse {
		0% { box-shadow: 0 0 0 rgba(139, 92, 246, 0); }
		20% { box-shadow: 0 0 5px rgba(139, 92, 246, 0.35); }
		45% { box-shadow: 0 0 0 rgba(139, 92, 246, 0); }
		100% { box-shadow: 0 0 0 rgba(139, 92, 246, 0); }
	}
</style>
