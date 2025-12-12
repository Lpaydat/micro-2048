<script lang="ts">
	import { getModalStore } from '@skeletonlabs/skeleton';
	import { MUSIC_TRACKS } from '$lib/game/rhythmEngine';
	import Music from 'lucide-svelte/icons/music';
	import Shuffle from 'lucide-svelte/icons/shuffle';
	import X from 'lucide-svelte/icons/x';

	const modalStore = getModalStore();

	// Sort tracks by BPM (slow to fast)
	const sortedTracks = [...MUSIC_TRACKS].sort((a, b) => a.bpm - b.bpm);

	// Get original index for each sorted track
	const getOriginalIndex = (track: typeof MUSIC_TRACKS[0]) => {
		return MUSIC_TRACKS.findIndex(t => t.name === track.name);
	};

	const selectTrack = (trackIndex: number | 'random') => {
		if ($modalStore[0]?.response) {
			$modalStore[0].response(trackIndex);
		}
		modalStore.close();
	};

	const cancel = () => {
		if ($modalStore[0]?.response) {
			$modalStore[0].response(null);
		}
		modalStore.close();
	};

	// BPM category helper
	const getBpmCategory = (bpm: number): string => {
		if (bpm <= 100) return 'Relaxed';
		if (bpm <= 120) return 'Medium';
		if (bpm <= 130) return 'Upbeat';
		return 'Fast';
	};

	const getBpmColor = (bpm: number): string => {
		if (bpm <= 100) return 'text-green-600';
		if (bpm <= 120) return 'text-blue-600';
		if (bpm <= 130) return 'text-amber-600';
		return 'text-red-600';
	};
</script>

<div class="card w-full max-w-md bg-[#FAF8EF] p-0 shadow-xl">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-purple-200 bg-purple-100 px-4 py-3">
		<h2 class="flex items-center gap-2 text-lg font-bold text-purple-900">
			<Music size={20} />
			Select Your Track
		</h2>
		<button
			type="button"
			onclick={cancel}
			class="rounded-full p-1 text-purple-600 transition-colors hover:bg-purple-200"
		>
			<X size={20} />
		</button>
	</div>

	<!-- Track List -->
	<div class="max-h-[60vh] overflow-y-auto p-3">
		<div class="flex flex-col gap-2">
			<!-- Random Option -->
			<button
				type="button"
				onclick={() => selectTrack('random')}
				class="group flex w-full items-center gap-3 rounded-lg border-2 border-purple-300 bg-gradient-to-r from-purple-50 to-violet-50 p-3 text-left transition-all hover:border-purple-500 hover:from-purple-100 hover:to-violet-100 hover:shadow-md"
			>
				<div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-purple-200 text-purple-700 transition-colors group-hover:bg-purple-300">
					<Shuffle size={20} />
				</div>
				<div class="flex-1">
					<div class="font-semibold text-purple-900">Random</div>
					<div class="text-xs text-purple-600">Let fate decide your rhythm</div>
				</div>
			</button>

			<!-- Divider -->
			<div class="my-1 flex items-center gap-2 px-2">
				<div class="h-px flex-1 bg-purple-200"></div>
				<span class="text-xs text-purple-400">or choose a track</span>
				<div class="h-px flex-1 bg-purple-200"></div>
			</div>

			<!-- Track Options (sorted by BPM) -->
			{#each sortedTracks as track}
				{@const originalIndex = getOriginalIndex(track)}
				{@const bpmCategory = getBpmCategory(track.bpm)}
				{@const bpmColor = getBpmColor(track.bpm)}
				<button
					type="button"
					onclick={() => selectTrack(originalIndex)}
					class="group flex w-full items-center gap-3 rounded-lg border-2 border-gray-200 bg-white p-3 text-left transition-all hover:border-purple-400 hover:bg-purple-50 hover:shadow-md"
				>
					<div class="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-full bg-gray-100 text-gray-600 transition-colors group-hover:bg-purple-200 group-hover:text-purple-700">
						<Music size={18} />
					</div>
					<div class="flex-1">
						<div class="font-semibold text-gray-800 group-hover:text-purple-900">{track.name}</div>
						<div class="text-xs text-gray-500">{bpmCategory}</div>
					</div>
					<div class="text-right">
						<div class="text-sm font-bold {bpmColor}">{track.bpm}</div>
						<div class="text-xs text-gray-400">BPM</div>
					</div>
				</button>
			{/each}
		</div>
	</div>

	<!-- Footer -->
	<div class="border-t border-gray-200 bg-gray-50 px-4 py-3">
		<button
			type="button"
			onclick={cancel}
			class="w-full rounded-lg border border-gray-300 bg-white py-2 text-sm font-medium text-gray-600 transition-colors hover:bg-gray-100"
		>
			Cancel
		</button>
	</div>
</div>
