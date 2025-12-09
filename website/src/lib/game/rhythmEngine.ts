/**
 * RhythmEngine - Industry-standard rhythm game timing system
 *
 * Architecture:
 * - Tone.Transport is the ONLY source of truth for timing
 * - All timing calculations derive from Transport.seconds
 * - Visual beat phase is a pure function of Transport time
 * - Input validation is a pure function of Transport time
 *
 * NO performance.now(), NO Date.now(), NO setInterval for timing
 */

import * as Tone from 'tone';

// ============================================================================
// TYPES
// ============================================================================

export interface RhythmSettings {
	enabled: boolean;
	bpm: number;
	tolerance: number; // milliseconds - window for valid hits
	useMusic?: boolean;
	trackIndex?: number | 'random'; // specific track index or 'random'
}

export interface RhythmFeedback {
	accuracy: 'perfect' | 'good' | 'early' | 'late' | 'miss';
	timingDiff: number; // milliseconds from nearest beat (signed: - = early, + = late)
	beatNumber: number;
	score: number;
}

export interface MusicTrack {
	name: string;
	url: string;
	bpm: number;
	firstBeatOffset: number; // seconds from start of file to first beat
}

// ============================================================================
// TRACK DATA
// ============================================================================

// BPM values verified with beat analyzer
// firstBeatOffset: time in seconds from file start to the first downbeat
// These need to be measured for each track!
// TODO: Measure these precisely with audio analysis
export const MUSIC_TRACKS: MusicTrack[] = [
	{ name: 'Watch Your Step', url: '/music/track1.mp3', bpm: 120, firstBeatOffset: 0 },
	{ name: 'Crypteque', url: '/music/track2.mp3', bpm: 130, firstBeatOffset: 0 },
	{ name: 'Tombtorial', url: '/music/track3.mp3', bpm: 100, firstBeatOffset: 0 }
];

// ============================================================================
// RHYTHM ENGINE
// ============================================================================

export class RhythmEngine {
	// Configuration
	private bpm: number;
	private tolerance: number;
	private useMusic: boolean;
	private trackIndex: number | 'random';

	// State
	private running: boolean = false;
	private currentTrack: MusicTrack | null = null;

	// Tone.js instances
	private player: Tone.Player | null = null;
	private metronomeHigh: Tone.Synth | null = null;
	private metronomeLow: Tone.Synth | null = null;
	private beatEventId: number | null = null;

	// Beat counter (for display only, not for timing)
	private beatCount: number = 0;

	// Calibration offset in seconds (user adjustment for audio latency)
	private calibrationOffset: number = 0;

	// First beat offset for current track (seconds from file start to first beat)
	private firstBeatOffset: number = 0;

	// Beat callback for visual sync debugging
	private onBeatCallback: ((beatNumber: number) => void) | null = null;

	constructor(settings: RhythmSettings) {
		this.bpm = settings.bpm;
		this.tolerance = settings.tolerance;
		this.useMusic = settings.useMusic ?? false;
		this.trackIndex = settings.trackIndex ?? 'random';
	}

	// ========================================================================
	// INITIALIZATION
	// ========================================================================

	/**
	 * Initialize audio system - MUST be called after user interaction
	 */
	async init(): Promise<void> {
		console.log(`🎵 [INIT] Starting with BPM: ${this.bpm}, useMusic: ${this.useMusic}`);

		// Start Tone.js audio context (requires user gesture)
		await Tone.start();
		console.log('🎵 [INIT] Audio context started');

		// Create metronome synths
		this.metronomeHigh = new Tone.Synth({
			oscillator: { type: 'triangle' },
			envelope: { attack: 0.001, decay: 0.1, sustain: 0, release: 0.1 }
		}).toDestination();
		this.metronomeHigh.volume.value = -6;

		this.metronomeLow = new Tone.Synth({
			oscillator: { type: 'triangle' },
			envelope: { attack: 0.001, decay: 0.1, sustain: 0, release: 0.1 }
		}).toDestination();
		this.metronomeLow.volume.value = -10;

		// Load music if enabled
		if (this.useMusic) {
			await this.loadTrack();
			console.log(`🎵 [INIT] After loadTrack, BPM is now: ${this.bpm}`);
		}

		// Load calibration from localStorage
		this.loadCalibration();

		console.log(`🎵 [INIT] Init complete. Final BPM: ${this.bpm}`);
	}

	/**
	 * Load music track (specific or random based on settings)
	 */
	private async loadTrack(): Promise<void> {
		let track: MusicTrack;

		if (this.trackIndex === 'random') {
			track = MUSIC_TRACKS[Math.floor(Math.random() * MUSIC_TRACKS.length)];
			console.log(`🎵 [LOAD] Random track selected: ${track.name}`);
		} else {
			const index = typeof this.trackIndex === 'number' ? this.trackIndex : parseInt(this.trackIndex);
			if (index >= 0 && index < MUSIC_TRACKS.length) {
				track = MUSIC_TRACKS[index];
				console.log(`🎵 [LOAD] Specific track selected: ${track.name} (index ${index})`);
			} else {
				console.warn(`🎵 [LOAD] Invalid track index ${index}, falling back to random`);
				track = MUSIC_TRACKS[Math.floor(Math.random() * MUSIC_TRACKS.length)];
			}
		}

		this.currentTrack = track;

		console.log(`🎵 [LOAD] Loading track: ${track.name} (track BPM: ${track.bpm})`);
		console.log(`🎵 [LOAD] BPM before load: ${this.bpm}`);

		try {
			this.player = new Tone.Player({
				url: track.url,
				loop: true
			}).toDestination();

			await Tone.loaded();

			// CRITICAL: Set BPM and first beat offset from track
			console.log(`🎵 [LOAD] Setting BPM from ${this.bpm} to ${track.bpm}`);
			this.bpm = track.bpm;
			this.firstBeatOffset = track.firstBeatOffset;

			console.log(`🎵 [LOAD] Loaded: ${track.name}, BPM: ${this.bpm}, firstBeatOffset: ${this.firstBeatOffset}s`);
		} catch (error) {
			console.error('🎵 [LOAD] Failed to load track:', error);
			this.useMusic = false;
			this.player = null;
		}
	}

	// ========================================================================
	// TRANSPORT CONTROL
	// ========================================================================

	/**
	 * Start the rhythm engine
	 */
	start(): void {
		if (this.running) return;

		const transport = Tone.getTransport();

		console.log(`🎵 [START] BPM before start: ${this.bpm}`);
		console.log(
			`🎵 [START] useMusic: ${this.useMusic}, currentTrack: ${this.currentTrack?.name}, player: ${!!this.player}`
		);

		// Reset state
		this.running = true;
		this.beatCount = 0;

		// If using music and track is loaded, use track BPM (override settings)
		if (this.useMusic && this.currentTrack && this.player) {
			console.log(
				`🎵 [START] Overriding BPM from ${this.bpm} to track BPM: ${this.currentTrack.bpm}`
			);
			this.bpm = this.currentTrack.bpm;
		}

		// Configure transport with our BPM
		transport.bpm.value = this.bpm;
		transport.position = 0;

		console.log(`🎵 [START] Final BPM: ${this.bpm}, Transport BPM: ${transport.bpm.value}`);

		// Schedule beat events
		// For music: schedule from firstBeatOffset so beats align with music
		// For metronome: schedule from 0
		const beatStartTime = this.useMusic && this.player ? this.firstBeatOffset : 0;
		
		this.beatEventId = transport.scheduleRepeat(
			(time) => {
				this.beatCount++;
				
				// Play metronome click only if not using music
				if (!this.useMusic || !this.player) {
					this.playMetronomeClick(time);
				}
				
				// Fire beat callback for visual sync
				if (this.onBeatCallback) {
					// Use Tone.Draw to sync callback with visual frame
					Tone.getDraw().schedule(() => {
						this.onBeatCallback?.(this.beatCount);
					}, time);
				}
			},
			'4n',
			beatStartTime
		);

		// Start music player synced to transport
		if (this.useMusic && this.player) {
			// Start music from beginning, but transport beats start at firstBeatOffset
			this.player.sync().start(0);
		}

		// Start the transport (this starts EVERYTHING)
		transport.start();

		console.log(`🎵 Started @ ${this.bpm} BPM (music: ${this.useMusic && !!this.player})`);
	}

	/**
	 * Stop the rhythm engine
	 */
	stop(): void {
		if (!this.running) return;

		const transport = Tone.getTransport();

		// Stop transport
		transport.stop();
		transport.position = 0;

		// Clear scheduled events
		if (this.beatEventId !== null) {
			transport.clear(this.beatEventId);
			this.beatEventId = null;
		}

		// Stop and unsync player
		if (this.player) {
			this.player.unsync();
			this.player.stop();
		}

		this.running = false;
		this.beatCount = 0;

		console.log('🎵 Stopped');
	}

	/**
	 * Clean up all resources
	 */
	dispose(): void {
		this.stop();

		this.player?.dispose();
		this.player = null;

		this.metronomeHigh?.dispose();
		this.metronomeHigh = null;

		this.metronomeLow?.dispose();
		this.metronomeLow = null;
	}

	// ========================================================================
	// METRONOME
	// ========================================================================

	private playMetronomeClick(time: number): void {
		// Note: beatCount is incremented in the scheduleRepeat callback, not here
		// Accent every 4 beats
		if (this.beatCount % 4 === 1) {
			this.metronomeHigh?.triggerAttackRelease('C5', '32n', time);
		} else {
			this.metronomeLow?.triggerAttackRelease('G4', '32n', time);
		}
	}

	// ========================================================================
	// TIMING - THE CORE
	// ========================================================================

	/**
	 * Get the current beat phase (0 to 1)
	 *
	 * This is THE function for visual sync.
	 * Call this every frame in requestAnimationFrame.
	 *
	 * Returns:
	 *   0.0 = exactly on a beat
	 *   0.5 = exactly between beats
	 *   approaching 1.0 = about to hit next beat
	 */
	getBeatPhase(): number {
		if (!this.running) return 0;

		const transport = Tone.getTransport();
		const beatLength = 60 / this.bpm;
		
		// Adjust for first beat offset and calibration
		// firstBeatOffset: when music beat actually starts in the file
		// calibrationOffset: user's device latency adjustment
		let seconds = transport.seconds - this.firstBeatOffset + this.calibrationOffset;
		
		// Handle negative time (before first beat)
		if (seconds < 0) {
			// Wrap to end of beat cycle
			seconds = beatLength + (seconds % beatLength);
		}

		// Position within current beat (0 to 1)
		const phase = (seconds % beatLength) / beatLength;

		return phase;
	}

	/**
	 * Get timing info for input validation
	 *
	 * Returns how far we are from the nearest beat.
	 * Negative = early (before beat), Positive = late (after beat)
	 */
	private getTimingFromNearestBeat(): { diffMs: number; isLate: boolean } {
		const transport = Tone.getTransport();
		const beatLength = 60 / this.bpm;
		
		// Adjust for first beat offset and calibration
		let seconds = transport.seconds - this.firstBeatOffset + this.calibrationOffset;
		
		// Handle negative time
		if (seconds < 0) {
			seconds = beatLength + (seconds % beatLength);
		}

		// Position within current beat (0 to beatLength in seconds)
		const positionInBeat = seconds % beatLength;

		// Distance to previous beat (0) vs next beat (beatLength)
		const distToPrev = positionInBeat;
		const distToNext = beatLength - positionInBeat;

		if (distToPrev <= distToNext) {
			// Closer to previous beat = we're LATE
			return { diffMs: distToPrev * 1000, isLate: true };
		} else {
			// Closer to next beat = we're EARLY
			return { diffMs: -distToNext * 1000, isLate: false };
		}
	}

	/**
	 * Check if player's input is on beat
	 *
	 * Call this when player makes a move.
	 * Returns feedback about their timing.
	 */
	checkRhythm(): RhythmFeedback {
		if (!this.running) {
			return { accuracy: 'perfect', timingDiff: 0, beatNumber: 0, score: 0 };
		}

		const { diffMs, isLate } = this.getTimingFromNearestBeat();
		const absDiff = Math.abs(diffMs);

		// Determine accuracy based on timing
		// Windows are relaxed for better feel (similar to Guitar Hero/NecroDancer casual)
		let accuracy: RhythmFeedback['accuracy'];
		let score: number;

		if (absDiff <= 75) {
			// Within 75ms = Perfect
			accuracy = 'perfect';
			score = 100;
		} else if (absDiff <= 150) {
			// Within 150ms = Good
			accuracy = 'good';
			score = 75;
		} else if (absDiff <= this.tolerance) {
			// Within tolerance = Early/Late
			accuracy = isLate ? 'late' : 'early';
			score = 50;
		} else {
			// Outside tolerance = Miss
			accuracy = 'miss';
			score = 0;
		}

		return {
			accuracy,
			timingDiff: diffMs, // Signed: negative = early, positive = late
			beatNumber: this.beatCount,
			score
		};
	}

	/**
	 * Simple check: is player currently in the hit window?
	 */
	isOnBeat(): boolean {
		if (!this.running) return true;

		const { diffMs } = this.getTimingFromNearestBeat();
		return Math.abs(diffMs) <= this.tolerance;
	}

	// ========================================================================
	// VISUAL HELPERS
	// ========================================================================

	/**
	 * Get all data needed for visual feedback
	 *
	 * Call this every frame for smooth animations.
	 */
	getVisualState(): {
		beatPhase: number; // 0-1, position in beat cycle
		intensity: number; // 0-1, how close to beat (1 = on beat)
		isInWindow: boolean; // true if player can hit now
		bpm: number;
		beatNumber: number;
	} {
		if (!this.running) {
			return {
				beatPhase: 0,
				intensity: 0,
				isInWindow: false,
				bpm: this.bpm,
				beatNumber: 0
			};
		}

		const beatPhase = this.getBeatPhase();

		// Calculate intensity: 1.0 at beat (phase near 0 or 1), 0 at middle
		// Using smooth cosine curve
		const distanceFromBeat = beatPhase <= 0.5 ? beatPhase : 1 - beatPhase;
		const intensity = Math.cos(distanceFromBeat * Math.PI);

		// Check if in hit window
		const { diffMs } = this.getTimingFromNearestBeat();
		const isInWindow = Math.abs(diffMs) <= this.tolerance;

		return {
			beatPhase,
			intensity: Math.max(0, intensity),
			isInWindow,
			bpm: this.bpm,
			beatNumber: this.beatCount
		};
	}

	// ========================================================================
	// CALIBRATION
	// ========================================================================

	private static readonly CALIBRATION_KEY = 'rhythm_calibration_offset';

	/**
	 * Load calibration offset from localStorage
	 */
	private loadCalibration(): void {
		if (typeof window === 'undefined') return;

		const stored = localStorage.getItem(RhythmEngine.CALIBRATION_KEY);
		if (stored) {
			const ms = parseFloat(stored);
			if (!isNaN(ms)) {
				this.calibrationOffset = ms / 1000;
				console.log(`🎵 [CALIBRATION] Loaded offset: ${ms}ms`);
			}
		}
	}

	/**
	 * Save calibration offset to localStorage
	 */
	static saveCalibration(ms: number): void {
		if (typeof window === 'undefined') return;
		localStorage.setItem(RhythmEngine.CALIBRATION_KEY, ms.toString());
		console.log(`🎵 [CALIBRATION] Saved offset: ${ms}ms`);
	}

	/**
	 * Get stored calibration offset (without needing an instance)
	 */
	static getStoredCalibration(): number {
		if (typeof window === 'undefined') return 0;
		const stored = localStorage.getItem(RhythmEngine.CALIBRATION_KEY);
		if (stored) {
			const ms = parseFloat(stored);
			if (!isNaN(ms)) return ms;
		}
		return 0;
	}

	/**
	 * Check if user has calibrated
	 */
	static hasCalibration(): boolean {
		if (typeof window === 'undefined') return false;
		return localStorage.getItem(RhythmEngine.CALIBRATION_KEY) !== null;
	}

	/**
	 * Set calibration offset (for audio latency compensation)
	 * Positive = audio is delayed, negative = audio is early
	 */
	setCalibrationOffset(ms: number): void {
		this.calibrationOffset = ms / 1000; // Convert to seconds
		console.log(`🎵 Calibration: ${ms}ms`);
	}

	getCalibrationOffset(): number {
		return this.calibrationOffset * 1000; // Return in ms
	}

	// ========================================================================
	// GETTERS
	// ========================================================================

	getBpm(): number {
		return this.bpm;
	}

	/**
	 * Debug: Log current state
	 */
	debugState(): void {
		const transport = Tone.getTransport();
		console.log('🎵 [DEBUG] RhythmEngine State:');
		console.log(`  - this.bpm: ${this.bpm}`);
		console.log(`  - Transport.bpm: ${transport.bpm.value}`);
		console.log(`  - running: ${this.running}`);
		console.log(`  - useMusic: ${this.useMusic}`);
		console.log(
			`  - currentTrack: ${this.currentTrack?.name || 'none'} (${this.currentTrack?.bpm || 'N/A'} BPM)`
		);
		console.log(`  - player loaded: ${!!this.player}`);
	}

	getTolerance(): number {
		return this.tolerance;
	}

	isRunning(): boolean {
		return this.running;
	}

	isUsingMusic(): boolean {
		return this.useMusic && this.player !== null;
	}

	getCurrentTrack(): MusicTrack | null {
		return this.currentTrack;
	}

	/**
	 * Set callback to be called on each beat (for visual debugging)
	 */
	setOnBeatCallback(callback: ((beatNumber: number) => void) | null): void {
		this.onBeatCallback = callback;
	}

	/**
	 * Get the first beat offset for current track
	 */
	getFirstBeatOffset(): number {
		return this.firstBeatOffset;
	}

	// ========================================================================
	// STATIC HELPERS
	// ========================================================================

	/**
	 * Parse rhythm settings from tournament description
	 * Format: [RHYTHM_MODE:true,BPM:120,TOLERANCE:150,MUSIC:true,TRACK:random]
	 * or: [RHYTHM_MODE:true,BPM:100,TOLERANCE:150,MUSIC:true,TRACK:2]
	 */
	static parseFromDescription(description: string): RhythmSettings | null {
		// Updated regex to handle TRACK parameter
		const match = description.match(
			/\[RHYTHM_MODE:true,BPM:(\d+),TOLERANCE:(\d+)(?:,MUSIC:(true|false))?(?:,TRACK:(\w+))?\]/
		);

		if (!match) return null;

		// Parse track index
		let trackIndex: number | 'random' = 'random';
		if (match[4] && match[4] !== 'random') {
			const parsed = parseInt(match[4], 10);
			if (!isNaN(parsed)) {
				trackIndex = parsed;
			}
		}

		return {
			enabled: true,
			bpm: parseInt(match[1], 10),
			tolerance: parseInt(match[2], 10),
			useMusic: match[3] === 'true',
			trackIndex
		};
	}
}
