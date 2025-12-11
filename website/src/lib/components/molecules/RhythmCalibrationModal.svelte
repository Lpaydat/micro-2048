<script lang="ts">
	import { onDestroy } from 'svelte';
	import { getModalStore } from '@skeletonlabs/skeleton';
	import * as Tone from 'tone';
	import { RhythmEngine } from '$lib/game/rhythmEngine.js';

	const modalStore = getModalStore();

	// Calibration settings
	const CALIBRATION_BPM = 100; // Fixed BPM for calibration
	const REQUIRED_TAPS = 16; // Number of taps needed
	const WARMUP_BEATS = 4; // Beats to wait before accepting taps

	// State
	let phase: 'intro' | 'calibrating' | 'complete' = $state('intro');
	let tapCount = $state(0);
	let beatCount = $state(0);
	let offsets: number[] = $state([]);
	let calculatedOffset = $state(0);
	let isPlaying = $state(false);

	// Visual state
	let beatPhase = $state(0);
	let showTapFeedback = $state(false);

	// Audio
	let metronomeSynth: Tone.Synth | null = null;
	let metronomeAccent: Tone.Synth | null = null;
	let beatEventId: number | null = null;
	let animationFrame: number;

	// Timing
	let beatLength = 60 / CALIBRATION_BPM; // seconds per beat

	async function startCalibration() {
		phase = 'calibrating';
		tapCount = 0;
		beatCount = 0;
		offsets = [];

		// Start audio context
		await Tone.start();

		const transport = Tone.getTransport();
		transport.bpm.value = CALIBRATION_BPM;
		transport.position = 0;

		// Create synths
		metronomeSynth = new Tone.Synth({
			oscillator: { type: 'triangle' },
			envelope: { attack: 0.001, decay: 0.1, sustain: 0, release: 0.1 }
		}).toDestination();
		metronomeSynth.volume.value = -6;

		metronomeAccent = new Tone.Synth({
			oscillator: { type: 'triangle' },
			envelope: { attack: 0.001, decay: 0.15, sustain: 0, release: 0.1 }
		}).toDestination();
		metronomeAccent.volume.value = -3;

		// Schedule metronome
		beatEventId = transport.scheduleRepeat(
			(time) => {
				beatCount++;
				// Accent every 4 beats
				if (beatCount % 4 === 1) {
					metronomeAccent?.triggerAttackRelease('C5', '16n', time);
				} else {
					metronomeSynth?.triggerAttackRelease('G4', '32n', time);
				}
			},
			'4n',
			0
		);

		transport.start();
		isPlaying = true;

		// Start animation loop
		animate();
	}

	function animate() {
		if (!isPlaying) return;

		const transport = Tone.getTransport();
		const seconds = transport.seconds;

		// Calculate beat phase (0 to 1)
		beatPhase = (seconds % beatLength) / beatLength;

		animationFrame = requestAnimationFrame(animate);
	}

	function handleTap() {
		if (phase !== 'calibrating' || !isPlaying) return;

		// Ignore taps during warmup
		if (beatCount < WARMUP_BEATS) {
			showTapFeedback = true;
			setTimeout(() => (showTapFeedback = false), 150);
			return;
		}

		const transport = Tone.getTransport();
		const seconds = transport.seconds;

		// Calculate offset from nearest beat
		const positionInBeat = seconds % beatLength;
		const distToPrev = positionInBeat;
		const distToNext = beatLength - positionInBeat;

		let offsetMs: number;
		if (distToPrev <= distToNext) {
			// Late (after beat)
			offsetMs = distToPrev * 1000;
		} else {
			// Early (before beat)
			offsetMs = -distToNext * 1000;
		}

		// Store offset (we want the pattern of how user taps)
		offsets.push(offsetMs);
		tapCount++;

		// Show visual feedback (just that we registered the tap)
		showTapFeedback = true;
		setTimeout(() => (showTapFeedback = false), 150);

		// Check if done
		if (tapCount >= REQUIRED_TAPS) {
			finishCalibration();
		}
	}

	function finishCalibration() {
		stopAudio();

		// Calculate median offset (more robust than average)
		const sorted = [...offsets].sort((a, b) => a - b);
		const mid = Math.floor(sorted.length / 2);
		calculatedOffset =
			sorted.length % 2 !== 0 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;

		// Round to nearest ms
		calculatedOffset = Math.round(calculatedOffset);

		// Save to localStorage
		RhythmEngine.saveCalibration(calculatedOffset);

		phase = 'complete';
	}

	function stopAudio() {
		isPlaying = false;

		if (animationFrame) {
			cancelAnimationFrame(animationFrame);
		}

		const transport = Tone.getTransport();
		transport.stop();
		transport.position = 0;

		if (beatEventId !== null) {
			transport.clear(beatEventId);
			beatEventId = null;
		}

		metronomeSynth?.dispose();
		metronomeSynth = null;

		metronomeAccent?.dispose();
		metronomeAccent = null;
	}

	function close() {
		stopAudio();
		// Trigger response callback before closing so parent can refresh
		if ($modalStore[0]?.response) {
			$modalStore[0].response(calculatedOffset);
		}
		modalStore.close();
	}

	onDestroy(() => {
		stopAudio();
	});

	// Handle keyboard tap
	function handleKeydown(event: KeyboardEvent) {
		if (event.code === 'Space' || event.code === 'Enter') {
			event.preventDefault();
			handleTap();
		}
	}
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="w-full max-w-md rounded-lg bg-[#1a1a2e] p-6 text-white shadow-xl">
	{#if phase === 'intro'}
		<!-- Introduction -->
		<div class="text-center">
			<div class="mb-4 text-4xl">🎵</div>
			<h2 class="mb-4 text-2xl font-bold text-purple-300">Rhythm Calibration</h2>
			<p class="mb-6 text-gray-300">
				Calibrate your audio latency for better rhythm accuracy. You'll hear beats - tap along to
				help us understand your device's timing.
			</p>

			<div class="mb-6 rounded-lg bg-purple-900/30 p-4 text-left">
				<h3 class="mb-2 font-semibold text-purple-200">How it works:</h3>
				<ol class="list-inside list-decimal space-y-1 text-sm text-gray-300">
					<li>Wait for 4 warm-up beats</li>
					<li>Tap the button (or press Space) on each beat</li>
					<li>Tap {REQUIRED_TAPS} times total</li>
					<li>We'll calculate your offset automatically</li>
				</ol>
			</div>

			<div class="flex gap-3">
				<button
					class="flex-1 rounded-lg bg-gray-600 py-3 font-semibold transition-colors hover:bg-gray-500"
					onclick={close}
				>
					Skip
				</button>
				<button
					class="flex-1 rounded-lg bg-purple-600 py-3 font-semibold transition-colors hover:bg-purple-500"
					onclick={startCalibration}
				>
					Start
				</button>
			</div>
		</div>
	{:else if phase === 'calibrating'}
		<!-- Calibration in progress -->
		<div class="text-center">
			<h2 class="mb-2 text-xl font-bold text-purple-300">
				{beatCount < WARMUP_BEATS ? 'Get Ready...' : 'Tap on the Beat!'}
			</h2>

			<!-- Progress -->
			<div class="mb-4">
				<div class="mb-1 text-sm text-gray-400">
					{#if beatCount < WARMUP_BEATS}
						Warm-up: {WARMUP_BEATS - beatCount} beats
					{:else}
						Taps: {tapCount} / {REQUIRED_TAPS}
					{/if}
				</div>
				<div class="h-2 overflow-hidden rounded-full bg-gray-700">
					<div
						class="h-full bg-purple-500 transition-all duration-150"
						style="width: {(tapCount / REQUIRED_TAPS) * 100}%"
					></div>
				</div>
			</div>

			<!-- Beat Visualizer -->
			<div class="relative mb-6 h-32 overflow-hidden rounded-lg bg-gray-800">
				<!-- Beat pulse ring -->
				<div
					class="absolute left-1/2 top-1/2 h-24 w-24 -translate-x-1/2 -translate-y-1/2 rounded-full border-4 border-purple-500 transition-all duration-75"
					style="transform: translate(-50%, -50%) scale({1 + (1 - beatPhase) * 0.5}); opacity: {1 - beatPhase * 0.7}"
				></div>

				<!-- Center dot -->
				<div
					class="absolute left-1/2 top-1/2 h-8 w-8 -translate-x-1/2 -translate-y-1/2 rounded-full bg-purple-500 transition-all duration-75"
					style="transform: translate(-50%, -50%) scale({beatPhase < 0.1 ? 1.5 : 1})"
				></div>

				<!-- Tap registered feedback -->
				{#if showTapFeedback}
					<div class="absolute left-1/2 top-3 -translate-x-1/2 text-lg font-bold text-purple-300">
						✓
					</div>
				{/if}
			</div>

			<!-- Tap button -->
			<button
				class="w-full rounded-lg bg-purple-600 py-6 text-xl font-bold transition-all hover:bg-purple-500 active:scale-95 active:bg-purple-400"
				onclick={handleTap}
				disabled={beatCount < WARMUP_BEATS}
			>
				{beatCount < WARMUP_BEATS ? 'Wait...' : 'TAP'}
			</button>

			<p class="mt-3 text-xs text-gray-500">Or press Space / Enter</p>
		</div>
	{:else if phase === 'complete'}
		<!-- Calibration complete -->
		<div class="text-center">
			<div class="mb-4 text-4xl">
				{Math.abs(calculatedOffset) < 30 ? '🎯' : '✅'}
			</div>
			<h2 class="mb-2 text-2xl font-bold text-green-400">Calibration Complete!</h2>

			<div class="mb-6 rounded-lg bg-gray-800 p-4">
				<div class="text-sm text-gray-400">Your offset</div>
				<div class="text-3xl font-bold text-white">
					{calculatedOffset > 0 ? '+' : ''}{calculatedOffset}ms
				</div>
				<div class="mt-2 text-sm text-gray-400">
					{#if Math.abs(calculatedOffset) < 30}
						Great! Your timing is nearly perfect.
					{:else if calculatedOffset > 0}
						You tend to tap slightly late. We'll compensate for this.
					{:else}
						You tend to tap slightly early. We'll compensate for this.
					{/if}
				</div>
			</div>

			<div class="flex gap-3">
				<button
					class="flex-1 rounded-lg bg-gray-600 py-3 font-semibold transition-colors hover:bg-gray-500"
					onclick={() => {
						phase = 'intro';
						tapCount = 0;
						offsets = [];
					}}
				>
					Retry
				</button>
				<button
					class="flex-1 rounded-lg bg-green-600 py-3 font-semibold transition-colors hover:bg-green-500"
					onclick={close}
				>
					Done
				</button>
			</div>
		</div>
	{/if}
</div>
