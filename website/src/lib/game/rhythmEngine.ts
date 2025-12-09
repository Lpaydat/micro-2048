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
}

// ============================================================================
// TRACK DATA
// ============================================================================

// BPM values verified with beat analyzer
export const MUSIC_TRACKS: MusicTrack[] = [
	{ name: 'Watch Your Step', url: '/music/track1.mp3', bpm: 120 },
	{ name: 'Crypteque', url: '/music/track2.mp3', bpm: 65 },
	{ name: 'Tombtorial', url: '/music/track3.mp3', bpm: 100 },
];

// ============================================================================
// RHYTHM ENGINE
// ============================================================================

export class RhythmEngine {
	// Configuration
	private bpm: number;
	private tolerance: number;
	private useMusic: boolean;
	
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

	constructor(settings: RhythmSettings) {
		this.bpm = settings.bpm;
		this.tolerance = settings.tolerance;
		this.useMusic = settings.useMusic ?? false;
	}

	// ========================================================================
	// INITIALIZATION
	// ========================================================================

	/**
	 * Initialize audio system - MUST be called after user interaction
	 */
	async init(): Promise<void> {
		// Start Tone.js audio context (requires user gesture)
		await Tone.start();
		console.log('🎵 Audio context started');
		
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
			await this.loadRandomTrack();
		}
	}

	/**
	 * Load a random music track
	 */
	private async loadRandomTrack(): Promise<void> {
		const track = MUSIC_TRACKS[Math.floor(Math.random() * MUSIC_TRACKS.length)];
		this.currentTrack = track;
		
		console.log(`🎵 Loading: ${track.name} (${track.bpm} BPM)`);
		
		try {
			this.player = new Tone.Player({
				url: track.url,
				loop: true,
			}).toDestination();
			
			await Tone.loaded();
			
			// CRITICAL: Set BPM from track
			this.bpm = track.bpm;
			
			console.log(`🎵 Loaded: ${track.name} @ ${track.bpm} BPM`);
		} catch (error) {
			console.error('🎵 Failed to load track:', error);
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
		
		// Reset state
		this.running = true;
		this.beatCount = 0;
		
		// If using music and track is loaded, use track BPM (override settings)
		if (this.useMusic && this.currentTrack && this.player) {
			this.bpm = this.currentTrack.bpm;
			console.log(`🎵 Using track BPM: ${this.bpm}`);
		}
		
		// Configure transport with our BPM
		transport.bpm.value = this.bpm;
		transport.position = 0;
		
		console.log(`🎵 Transport BPM set to: ${transport.bpm.value}`);
		
		// Schedule metronome clicks on each beat (only if not using music)
		if (!this.useMusic || !this.player) {
			this.beatEventId = transport.scheduleRepeat((time) => {
				this.playMetronomeClick(time);
			}, '4n', 0);
		}
		
		// Start music player synced to transport
		if (this.useMusic && this.player) {
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
		this.beatCount++;
		
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
		const seconds = transport.seconds + this.calibrationOffset;
		const beatLength = 60 / this.bpm;
		
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
		const seconds = transport.seconds + this.calibrationOffset;
		const beatLength = 60 / this.bpm;
		
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
		beatPhase: number;      // 0-1, position in beat cycle
		intensity: number;      // 0-1, how close to beat (1 = on beat)
		isInWindow: boolean;    // true if player can hit now
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
		const distanceFromBeat = beatPhase <= 0.5 ? beatPhase : (1 - beatPhase);
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

	// ========================================================================
	// STATIC HELPERS
	// ========================================================================

	/**
	 * Parse rhythm settings from tournament description
	 * Format: [RHYTHM_MODE:true,BPM:120,TOLERANCE:150,MUSIC:true]
	 */
	static parseFromDescription(description: string): RhythmSettings | null {
		const match = description.match(
			/\[RHYTHM_MODE:true,BPM:(\d+),TOLERANCE:(\d+)(?:,MUSIC:(true|false))?\]/
		);
		
		if (!match) return null;
		
		return {
			enabled: true,
			bpm: parseInt(match[1], 10),
			tolerance: parseInt(match[2], 10),
			useMusic: match[3] === 'true'
		};
	}
}
