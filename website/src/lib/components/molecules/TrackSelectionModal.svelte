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

<div class="w-full max-w-md overflow-hidden rounded-2xl bg-gradient-to-b from-slate-900 to-slate-950 shadow-2xl">
	<!-- Header -->
	<div class="px-5 pb-2 pt-5">
		<div class="flex items-center justify-between">
			<h2 class="flex items-center gap-2.5 text-lg font-semibold text-white">
				<div class="flex h-8 w-8 items-center justify-center rounded-lg bg-purple-500/20">
					<Music size={18} class="text-purple-400" />
				</div>
				Select Your Track
			</h2>
			<button
				type="button"
				onclick={cancel}
				class="flex h-8 w-8 items-center justify-center rounded-lg text-slate-400 transition-all hover:bg-white/10 hover:text-white"
			>
				<X size={18} />
			</button>
		</div>
	</div>

	<!-- Track List -->
	<div class="max-h-[60vh] overflow-y-auto px-3 pb-3">
		<div class="flex flex-col gap-1.5">
			<!-- Random Option -->
			<button
				type="button"
				onclick={() => selectTrack('random')}
				class="group flex w-full items-center gap-3 rounded-xl bg-gradient-to-r from-purple-500/10 to-violet-500/10 p-3 text-left transition-all hover:from-purple-500/20 hover:to-violet-500/20 hover:shadow-lg hover:shadow-purple-500/10 active:scale-[0.98]"
			>
				<div class="flex h-11 w-11 flex-shrink-0 items-center justify-center rounded-xl bg-gradient-to-br from-purple-500 to-violet-600 text-white shadow-lg shadow-purple-500/30 transition-transform group-hover:scale-105">
					<Shuffle size={20} />
				</div>
				<div class="flex-1">
					<div class="font-semibold text-white">Random</div>
					<div class="text-xs text-slate-400">Let fate decide your rhythm</div>
				</div>
				<div class="rounded-lg bg-purple-500/20 px-2.5 py-1 text-xs font-medium text-purple-300">
					Surprise me
				</div>
			</button>

			<!-- Divider -->
			<div class="my-2 flex items-center gap-3 px-2">
				<div class="h-px flex-1 bg-gradient-to-r from-transparent via-slate-700 to-transparent"></div>
				<span class="text-xs font-medium text-slate-500">or choose a track</span>
				<div class="h-px flex-1 bg-gradient-to-r from-transparent via-slate-700 to-transparent"></div>
			</div>

			<!-- Track Options (sorted by BPM) -->
			{#each sortedTracks as track}
				{@const originalIndex = getOriginalIndex(track)}
				{@const bpmCategory = getBpmCategory(track.bpm)}
				{@const bpmColor = getBpmColor(track.bpm)}
				{@const bgColor = track.bpm <= 100 ? 'from-green-500/10 to-emerald-500/10 hover:from-green-500/20 hover:to-emerald-500/20 hover:shadow-green-500/10' : 
					track.bpm <= 120 ? 'from-blue-500/10 to-cyan-500/10 hover:from-blue-500/20 hover:to-cyan-500/20 hover:shadow-blue-500/10' : 
					track.bpm <= 130 ? 'from-amber-500/10 to-orange-500/10 hover:from-amber-500/20 hover:to-orange-500/20 hover:shadow-amber-500/10' : 
					'from-red-500/10 to-rose-500/10 hover:from-red-500/20 hover:to-rose-500/20 hover:shadow-red-500/10'}
				{@const iconBg = track.bpm <= 100 ? 'from-green-500 to-emerald-600 shadow-green-500/30' : 
					track.bpm <= 120 ? 'from-blue-500 to-cyan-600 shadow-blue-500/30' : 
					track.bpm <= 130 ? 'from-amber-500 to-orange-600 shadow-amber-500/30' : 
					'from-red-500 to-rose-600 shadow-red-500/30'}
				{@const bpmTextColor = track.bpm <= 100 ? 'text-green-400' : 
					track.bpm <= 120 ? 'text-blue-400' : 
					track.bpm <= 130 ? 'text-amber-400' : 
					'text-red-400'}
				<button
					type="button"
					onclick={() => selectTrack(originalIndex)}
					class="group flex w-full items-center gap-3 rounded-xl bg-gradient-to-r {bgColor} p-3 text-left transition-all hover:shadow-lg active:scale-[0.98]"
				>
					<div class="flex h-11 w-11 flex-shrink-0 items-center justify-center rounded-xl bg-gradient-to-br {iconBg} text-white shadow-lg transition-transform group-hover:scale-105">
						<Music size={18} />
					</div>
					<div class="flex-1 min-w-0">
						<div class="font-semibold text-white truncate">{track.name}</div>
						<div class="text-xs text-slate-400">{bpmCategory}</div>
					</div>
					<div class="flex flex-col items-end">
						<div class="text-lg font-bold {bpmTextColor}">{track.bpm}</div>
						<div class="text-[10px] font-medium uppercase tracking-wider text-slate-500">BPM</div>
					</div>
				</button>
			{/each}
		</div>
	</div>

	<!-- Footer -->
	<div class="px-3 pb-4">
		<button
			type="button"
			onclick={cancel}
			class="w-full rounded-xl bg-white/5 py-2.5 text-sm font-medium text-slate-400 transition-all hover:bg-white/10 hover:text-white active:scale-[0.98]"
		>
			Cancel
		</button>
	</div>
</div>
