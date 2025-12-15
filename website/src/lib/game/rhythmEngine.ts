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
	trackIndex?: number | 'random' | 'selectable'; // specific track index, 'random' (forced), or 'selectable' (player choice)
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
	{ name: 'Tombtorial', url: '/music/track3.mp3', bpm: 100, firstBeatOffset: 0 },
	{ name: 'Disco Descent', url: '/music/track4.mp3', bpm: 115, firstBeatOffset: 0 },
	{ name: 'Mausoleum Mash', url: '/music/track5.mp3', bpm: 140, firstBeatOffset: 0 }
];

// ============================================================================
// RHYTHM ENGINE
// ============================================================================

export class RhythmEngine {
	// Configuration
	private bpm: number;
	private tolerance: number;
	private useMusic: boolean;
	private trackIndex: number | 'random' | 'selectable';

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

	// Music duration for loop handling
	private musicDuration: number = 0;

	// REMOVED: VISUAL_PHASE_ADVANCE was a magic number measured on one device
	// Now we rely ONLY on calibrationOffset (user-measured) for timing adjustment
	// This is cleaner and more accurate across different devices/browsers

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
		// Start Tone.js audio context (requires user gesture)
		await Tone.start();

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
		}

		// Load calibration from localStorage
		this.loadCalibration();
	}

	/**
	 * Load music track (specific or random based on settings)
	 */
	private async loadTrack(): Promise<void> {
		let track: MusicTrack;

		if (this.trackIndex === 'random') {
			track = MUSIC_TRACKS[Math.floor(Math.random() * MUSIC_TRACKS.length)];
		} else {
			const index = typeof this.trackIndex === 'number' ? this.trackIndex : parseInt(this.trackIndex);
			if (index >= 0 && index < MUSIC_TRACKS.length) {
				track = MUSIC_TRACKS[index];
			} else {
				console.warn(`Invalid track index ${index}, falling back to random`);
				track = MUSIC_TRACKS[Math.floor(Math.random() * MUSIC_TRACKS.length)];
			}
		}

		this.currentTrack = track;

		try {
			this.player = new Tone.Player({
				url: track.url,
				loop: true
			}).toDestination();

			await Tone.loaded();

			// Set BPM and first beat offset from track
			this.bpm = track.bpm;
			this.firstBeatOffset = track.firstBeatOffset;
			
			// Store music duration for loop handling
			this.musicDuration = this.player.buffer.duration;
		} catch (error) {
			console.error('Failed to load track:', error);
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
		}

		// Configure transport with our BPM
		transport.bpm.value = this.bpm;
		transport.position = 0;

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
	 * Get effective time within music loop
	 * 
	 * When music loops, transport keeps counting but music restarts.
	 * This returns the time position relative to the current loop iteration.
	 */
	private getEffectiveTime(): number {
		const transport = Tone.getTransport();
		let seconds = transport.seconds;
		
		// If using music with a known duration, wrap time to account for loops
		if (this.useMusic && this.player && this.musicDuration > 0) {
			seconds = seconds % this.musicDuration;
		}
		
		return seconds;
	}

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

		const beatLength = 60 / this.bpm;
		
		// Get effective time (accounts for music loops)
		// Adjust for:
		// - firstBeatOffset: when music beat actually starts in the file
		// - calibrationOffset: user's device latency adjustment (the ONLY timing adjustment)
		//   SUBTRACT calibration because: if user taps late (positive offset), we need to
		//   shift the visual earlier (subtract from time) so beats align with when they hear it
		let seconds = this.getEffectiveTime() - this.firstBeatOffset - this.calibrationOffset;
		
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
		const beatLength = 60 / this.bpm;
		
		// Get effective time (accounts for music loops)
		// Adjust for first beat offset and calibration
		// SUBTRACT calibration: if user taps late (positive offset), shift window earlier
		let seconds = this.getEffectiveTime() - this.firstBeatOffset - this.calibrationOffset;
		
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

		// Determine accuracy based on timing (relative to tolerance setting)
		// Perfect: within 50% of tolerance
		// Good: within 100% of tolerance
		// Miss: outside tolerance
		const perfectWindow = this.tolerance * 0.5;
		
		let accuracy: RhythmFeedback['accuracy'];
		let score: number;

		if (absDiff <= perfectWindow) {
			// Within 50% of tolerance = Perfect
			accuracy = 'perfect';
			score = 100;
		} else if (absDiff <= this.tolerance) {
			// Within tolerance = Good (early/late indicator)
			accuracy = isLate ? 'late' : 'early';
			score = 75;
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
			}
		}
	}

	/**
	 * Save calibration offset to localStorage
	 */
	static saveCalibration(ms: number): void {
		if (typeof window === 'undefined') return;
		localStorage.setItem(RhythmEngine.CALIBRATION_KEY, ms.toString());
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
	 * Debug: Get current state (for debugging)
	 */
	debugState(): {
		bpm: number;
		transportBpm: number;
		running: boolean;
		useMusic: boolean;
		currentTrack: string;
		playerLoaded: boolean;
	} {
		const transport = Tone.getTransport();
		return {
			bpm: this.bpm,
			transportBpm: transport.bpm.value,
			running: this.running,
			useMusic: this.useMusic,
			currentTrack: this.currentTrack?.name || 'none',
			playerLoaded: !!this.player
		};
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
	// Regex pattern for rhythm settings tag
	private static readonly RHYTHM_TAG_REGEX = /\s*\[RHYTHM_MODE:true,BPM:(\d+),TOLERANCE:(\d+)(?:,MUSIC:(true|false))?(?:,TRACK:(\w+))?\]/;

	/**
	 * Parse rhythm settings from tournament description
	 * Format: [RHYTHM_MODE:true,BPM:120,TOLERANCE:150,MUSIC:true,TRACK:random]
	 * or: [RHYTHM_MODE:true,BPM:100,TOLERANCE:150,MUSIC:true,TRACK:2]
	 */
	static parseFromDescription(description: string): RhythmSettings | null {
		const match = description.match(RhythmEngine.RHYTHM_TAG_REGEX);

		if (!match) return null;

		// Parse track index: 'random', 'selectable', or a number
		let trackIndex: number | 'random' | 'selectable' = 'random';
		if (match[4]) {
			if (match[4] === 'selectable') {
				trackIndex = 'selectable';
			} else if (match[4] !== 'random') {
				const parsed = parseInt(match[4], 10);
				if (!isNaN(parsed)) {
					trackIndex = parsed;
				}
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

	/**
	 * Check if description contains rhythm mode settings
	 */
	static isRhythmMode(description: string | undefined | null): boolean {
		if (!description) return false;
		return description.includes('[RHYTHM_MODE:true');
	}

	/**
	 * Remove rhythm settings tag from description for display
	 * Returns the clean description text without the machine-readable tag
	 */
	static cleanDescription(description: string | undefined | null): string {
		if (!description) return '';
		return description.replace(RhythmEngine.RHYTHM_TAG_REGEX, '').trim();
	}

	/**
	 * Get rhythm settings display info for UI
	 * Returns null if not rhythm mode, otherwise returns formatted display info
	 */
	static getDisplayInfo(description: string | undefined | null): {
		bpm: number;
		tolerance: number;
		useMusic: boolean;
		trackIndex: number | 'random' | 'selectable';
		trackName: string | null;
	} | null {
		if (!description) return null;
		
		const settings = RhythmEngine.parseFromDescription(description);
		if (!settings) return null;

		// Get track name if specific track selected
		let trackName: string | null = null;
		if (typeof settings.trackIndex === 'number' && MUSIC_TRACKS[settings.trackIndex]) {
			trackName = MUSIC_TRACKS[settings.trackIndex].name;
		}

		return {
			bpm: settings.bpm,
			tolerance: settings.tolerance,
			useMusic: settings.useMusic ?? false,
			trackIndex: settings.trackIndex ?? 'random',
			trackName
		};
	}

	/**
	 * Check if track selection is player's choice
	 */
	static isTrackSelectable(description: string | undefined | null): boolean {
		if (!description) return false;
		const settings = RhythmEngine.parseFromDescription(description);
		return settings?.trackIndex === 'selectable';
	}
}
