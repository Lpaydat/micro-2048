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
	const isRhythmMode = $derived(description?.includes('[RHYTHM_MODE:true') ?? false);
	
	// Extract rhythm settings from description
	const rhythmSettings = $derived(() => {
		if (!isRhythmMode || !description) return null;
		const match = description.match(/\[RHYTHM_MODE:true,BPM:(\d+),TOLERANCE:(\d+),MUSIC:(true|false)\]/);
		if (match) {
			return {
				bpm: parseInt(match[1]),
				tolerance: parseInt(match[2]),
				useMusic: match[3] === 'true'
			};
		}
		return null;
	});

	// Clean description (remove rhythm settings tag)
	const cleanDescription = $derived(
		description?.replace(/\s*\[RHYTHM_MODE:true,BPM:\d+,TOLERANCE:\d+,MUSIC:(true|false)\]/, '').trim() || ''
	);

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
						<span class="inline-flex items-center rounded-md bg-violet-100 px-2 py-1 text-xs font-medium text-violet-700 ring-1 ring-inset ring-violet-300">
							{settings?.bpm} BPM
						</span>
						<span class="inline-flex items-center rounded-md bg-violet-100 px-2 py-1 text-xs font-medium text-violet-700 ring-1 ring-inset ring-violet-300">
							±{settings?.tolerance}ms
						</span>
						{#if settings?.useMusic}
							<span class="inline-flex items-center gap-1 rounded-md bg-violet-100 px-2 py-1 text-xs font-medium text-violet-700 ring-1 ring-inset ring-violet-300">
								<Music size={10} />
								Music
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
	/* Rhythm mode hover effects - 120 BPM beat (500ms per beat) */
	:global(.rhythm-item) {
		transition: all 0.2s ease;
		position: relative;
		overflow: hidden;
	}

	:global(.rhythm-item::before) {
		content: '';
		position: absolute;
		top: 0;
		left: -100%;
		width: 100%;
		height: 100%;
		background: linear-gradient(
			90deg,
			transparent,
			rgba(139, 92, 246, 0.2),
			transparent
		);
		transition: left 0.5s ease;
		pointer-events: none;
		z-index: 1;
	}

	:global(.rhythm-item:hover) {
		background: linear-gradient(to bottom right, rgb(221, 214, 254), rgb(233, 213, 255)) !important;
		border-color: rgb(139, 92, 246) !important;
		animation: rhythm-beat 500ms ease-in-out infinite;
	}

	:global(.rhythm-item:hover::before) {
		left: 100%;
	}

	/* Main beat animation - scale + glow pulse at 120 BPM */
	@keyframes rhythm-beat {
		0%, 100% {
			transform: scale(1);
			box-shadow: 
				0 0 15px rgba(139, 92, 246, 0.3),
				0 0 30px rgba(139, 92, 246, 0.15),
				inset 0 0 15px rgba(139, 92, 246, 0.05);
		}
		50% {
			transform: scale(1.015);
			box-shadow: 
				0 0 25px rgba(139, 92, 246, 0.6),
				0 0 50px rgba(139, 92, 246, 0.3),
				inset 0 0 25px rgba(139, 92, 246, 0.1);
		}
	}

	/* Music icon bounce synced to beat */
	:global(.rhythm-item:hover [data-lucide="music"]) {
		animation: music-bounce 500ms ease-in-out infinite;
	}

	@keyframes music-bounce {
		0%, 100% { 
			transform: scale(1) rotate(0deg); 
		}
		25% {
			transform: scale(1.2) rotate(-5deg);
		}
		50% { 
			transform: scale(1) rotate(0deg); 
		}
		75% {
			transform: scale(1.2) rotate(5deg);
		}
	}

	/* Rhythm badge pulse */
	:global(.rhythm-item:hover .badge) {
		animation: badge-pulse 500ms ease-in-out infinite;
	}

	@keyframes badge-pulse {
		0%, 100% { 
			transform: scale(1);
			opacity: 1;
		}
		50% { 
			transform: scale(1.05);
			opacity: 0.9;
		}
	}

	/* BPM/tolerance tags subtle pulse */
	:global(.rhythm-item:hover .ring-violet-300) {
		animation: tag-glow 500ms ease-in-out infinite;
	}

	@keyframes tag-glow {
		0%, 100% {
			box-shadow: 0 0 0 rgba(139, 92, 246, 0);
		}
		50% {
			box-shadow: 0 0 8px rgba(139, 92, 246, 0.5);
		}
	}
</style>
