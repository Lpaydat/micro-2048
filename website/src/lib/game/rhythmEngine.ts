import * as Tone from 'tone';

// Get the Transport singleton (getTransport() is deprecated)
const getTransport = () => Tone.getTransport();

export interface RhythmSettings {
	enabled: boolean;
	bpm: number;
	tolerance: number; // milliseconds
	useMusic?: boolean; // Use music instead of metronome
}

export interface RhythmFeedback {
	accuracy: 'perfect' | 'good' | 'miss' | 'early' | 'late';
	timingDiff: number; // milliseconds from beat
	beatNumber: number;
	score: number;
}

export interface MusicTrack {
	name: string;
	url: string;
	bpm: number;
	// Offset in seconds from start to first beat (for tracks that don't start on beat)
	firstBeatOffset: number;
}

// Music tracks from Crypt of the NecroDancer OST
// BPM values from beat analyzer (actual audio file analysis)
export const MUSIC_TRACKS: MusicTrack[] = [
	{ name: 'Watch Your Step (Training)', url: '/music/track1.mp3', bpm: 120, firstBeatOffset: 0 },
	{ name: 'Crypteque (1-2)', url: '/music/track2.mp3', bpm: 65, firstBeatOffset: 0 },
	{ name: 'Tombtorial (Tutorial)', url: '/music/track3.mp3', bpm: 100, firstBeatOffset: 0 },
];

/**
 * RhythmEngine - Tone.js based rhythm game engine
 * 
 * Uses getTransport() for sample-accurate beat timing.
 * All timing is derived from the audio context, not JS timers.
 */
export class RhythmEngine {
	private settings: RhythmSettings;
	private isRunning: boolean = false;
	private currentBeat: number = 0;
	
	// Music playback
	private player: Tone.Player | null = null;
	private currentTrack: MusicTrack | null = null;
	private musicLoaded: boolean = false;
	
	// Metronome
	private metronome: Tone.Synth | null = null;
	private metronomeLowSynth: Tone.Synth | null = null;
	
	// Beat scheduling
	private beatEventId: number | null = null;
	
	// Calibration
	private userCalibrationOffset: number = 0; // ms
	
	// Auto-calibration
	private autoCalibrationEnabled: boolean = true;
	private autoCalibrationSamples: number[] = [];
	private readonly AUTO_CALIBRATION_MIN_SAMPLES = 10;
	private readonly AUTO_CALIBRATION_MAX_SAMPLES = 30;
	private readonly AUTO_CALIBRATION_MAX_OFFSET = 100;
	private onCalibrationChangeCallback: ((offset: number) => void) | null = null;

	constructor(settings: RhythmSettings) {
		this.settings = settings;
	}

	/**
	 * Initialize audio - must be called after user interaction (browser requirement)
	 */
	async initAudio(): Promise<void> {
		// Start Tone.js audio context (requires user gesture)
		await Tone.start();
		console.log('🎵 Tone.js audio context started');
		
		// Set up metronome synths
		this.metronome = new Tone.Synth({
			oscillator: { type: 'sine' },
			envelope: { attack: 0.001, decay: 0.1, sustain: 0, release: 0.1 }
		}).toDestination();
		this.metronome.volume.value = -10;
		
		this.metronomeLowSynth = new Tone.Synth({
			oscillator: { type: 'sine' },
			envelope: { attack: 0.001, decay: 0.1, sustain: 0, release: 0.1 }
		}).toDestination();
		this.metronomeLowSynth.volume.value = -10;
		
		// Load music if enabled
		if (this.settings.useMusic) {
			await this.loadRandomTrack();
		}
	}

	/**
	 * Load a random music track
	 */
	private async loadRandomTrack(): Promise<void> {
		const randomIndex = Math.floor(Math.random() * MUSIC_TRACKS.length);
		this.currentTrack = MUSIC_TRACKS[randomIndex];
		
		console.log(`🎵 Loading track: ${this.currentTrack.name}...`);
		
		try {
			// Create Tone.Player for music playback
			this.player = new Tone.Player({
				url: this.currentTrack.url,
				loop: true,
				onload: () => {
					console.log(`🎵 Track loaded: ${this.currentTrack?.name}`);
					this.musicLoaded = true;
				}
			}).toDestination();
			
			// Wait for the player to load
			await Tone.loaded();
			
			// Set BPM from track
			this.settings.bpm = this.currentTrack.bpm;
			getTransport().bpm.value = this.currentTrack.bpm;
			
			console.log(`🎵 Ready: ${this.currentTrack.name} @ ${this.currentTrack.bpm} BPM`);
			
		} catch (error) {
			console.warn('🎵 Failed to load music, will use metronome:', error);
			this.settings.useMusic = false;
			this.musicLoaded = false;
		}
	}

	/**
	 * Start the rhythm engine
	 */
	start(): void {
		if (!this.settings.enabled || this.isRunning) return;
		
		this.isRunning = true;
		this.currentBeat = 0;
		this.autoCalibrationSamples = [];
		
		// Determine BPM: use track BPM for music, settings BPM for metronome
		const effectiveBpm = (this.settings.useMusic && this.musicLoaded && this.currentTrack)
			? this.currentTrack.bpm
			: this.settings.bpm;
		
		// Update settings to reflect actual BPM being used
		this.settings.bpm = effectiveBpm;
		
		// Configure Transport with the correct BPM
		getTransport().bpm.value = effectiveBpm;
		getTransport().position = 0;
		
		// Schedule beat events - fires exactly on each beat
		this.beatEventId = getTransport().scheduleRepeat((time) => {
			this.onBeat(time);
		}, '4n'); // quarter note = one beat
		
		// Start transport
		getTransport().start();
		
		// Start music or metronome
		if (this.settings.useMusic && this.player && this.musicLoaded) {
			// Sync player to transport
			this.player.sync().start(0);
			console.log(`🎵 Playing: ${this.currentTrack?.name} @ ${effectiveBpm} BPM`);
		} else {
			console.log(`🎵 Metronome started @ ${effectiveBpm} BPM`);
		}
	}

	/**
	 * Called exactly on each beat by getTransport()
	 */
	private onBeat(time: number): void {
		this.currentBeat++;
		
		// Play metronome click if not using music
		if (!this.settings.useMusic || !this.musicLoaded) {
			// Accent on beat 1 of each measure (every 4 beats)
			if (this.currentBeat % 4 === 1) {
				this.metronome?.triggerAttackRelease('C5', '32n', time);
			} else {
				this.metronomeLowSynth?.triggerAttackRelease('G4', '32n', time);
			}
		}
	}

	/**
	 * Stop the rhythm engine
	 */
	stop(): void {
		this.isRunning = false;
		
		// Stop and reset transport
		getTransport().stop();
		getTransport().position = 0;
		
		// Clear beat event
		if (this.beatEventId !== null) {
			getTransport().clear(this.beatEventId);
			this.beatEventId = null;
		}
		
		// Stop player
		if (this.player) {
			this.player.unsync();
			this.player.stop();
		}
		
		// Reset state
		this.currentBeat = 0;
		this.autoCalibrationSamples = [];
	}

	/**
	 * Clean up resources
	 */
	dispose(): void {
		this.stop();
		
		this.player?.dispose();
		this.player = null;
		
		this.metronome?.dispose();
		this.metronome = null;
		
		this.metronomeLowSynth?.dispose();
		this.metronomeLowSynth = null;
		
		this.musicLoaded = false;
	}

	/**
	 * Check if a move is on rhythm - THE CORE RHYTHM CHECK
	 * 
	 * Uses getTransport().seconds for sample-accurate timing
	 * @param _timestamp Optional timestamp parameter (ignored, kept for API compatibility)
	 */
	checkRhythm(_timestamp?: number): RhythmFeedback {
		if (!this.settings.enabled || !this.isRunning) {
			return {
				accuracy: 'perfect',
				timingDiff: 0,
				beatNumber: 0,
				score: 0
			};
		}

		// Get current position in the beat cycle from Transport
		// This is THE source of truth - derived directly from audio context
		const transportSeconds = getTransport().seconds;
		const beatLengthSeconds = 60 / this.settings.bpm;
		
		// Apply calibration offset (convert ms to seconds)
		const calibrationSeconds = this.userCalibrationOffset / 1000;
		const adjustedTime = transportSeconds - calibrationSeconds;
		
		// Calculate position within current beat (0 to 1)
		const beatPosition = (adjustedTime % beatLengthSeconds) / beatLengthSeconds;
		
		// Calculate distance from nearest beat
		// beatPosition near 0 = just after beat, near 1 = just before next beat
		// We want distance from the nearest beat edge (0 or 1)
		let distanceFromBeat: number;
		let isLate: boolean;
		
		if (beatPosition <= 0.5) {
			// Closer to the beat that just happened (late hit)
			distanceFromBeat = beatPosition;
			isLate = true;
		} else {
			// Closer to the upcoming beat (early hit)
			distanceFromBeat = 1 - beatPosition;
			isLate = false;
		}
		
		// Convert to milliseconds
		const timingDiffMs = distanceFromBeat * beatLengthSeconds * 1000;
		
		// Signed timing for auto-calibration (positive = late, negative = early)
		const signedTimingMs = isLate ? timingDiffMs : -timingDiffMs;
		
		// Determine accuracy
		let accuracy: RhythmFeedback['accuracy'];
		let score: number;
		
		if (timingDiffMs <= 50) {
			accuracy = 'perfect';
			score = 100;
		} else if (timingDiffMs <= 100) {
			accuracy = 'good';
			score = 50;
		} else if (timingDiffMs <= this.settings.tolerance) {
			accuracy = isLate ? 'late' : 'early';
			score = 25;
		} else {
			accuracy = 'miss';
			score = 0;
		}
		
		// Record for auto-calibration
		this.recordHitForAutoCalibration(signedTimingMs, accuracy);

		return {
			accuracy,
			timingDiff: timingDiffMs,
			beatNumber: this.currentBeat,
			score
		};
	}

	/**
	 * Get current beat phase (0 to 1) for visual animation
	 * 
	 * This is called every frame and returns the exact position in the beat cycle
	 * derived from getTransport() - guarantees perfect visual sync
	 */
	getBeatPhase(): number {
		if (!this.isRunning) return 0;
		
		const transportSeconds = getTransport().seconds;
		const beatLengthSeconds = 60 / this.settings.bpm;
		
		// Apply calibration
		const calibrationSeconds = this.userCalibrationOffset / 1000;
		const adjustedTime = transportSeconds - calibrationSeconds;
		
		// Return position in beat cycle (0 to 1)
		return (adjustedTime % beatLengthSeconds) / beatLengthSeconds;
	}

	/**
	 * Get visual feedback data for UI components
	 */
	getVisualFeedback(): {
		isOnBeat: boolean;
		beatProgress: number;
		timeToNext: number;
		intensity: number;
	} {
		if (!this.settings.enabled || !this.isRunning) {
			return {
				isOnBeat: false,
				beatProgress: 0,
				timeToNext: 0,
				intensity: 0
			};
		}

		const beatPhase = this.getBeatPhase();
		const beatLengthMs = (60 / this.settings.bpm) * 1000;
		
		// Distance from nearest beat edge (0 or 1)
		const distanceFromBeat = beatPhase <= 0.5 ? beatPhase : (1 - beatPhase);
		const timingDiffMs = distanceFromBeat * beatLengthMs;
		
		// Time to next beat
		const timeToNext = (1 - beatPhase) * beatLengthMs;
		
		// Is on beat (within tolerance)?
		const isOnBeat = timingDiffMs <= this.settings.tolerance;
		
		// Intensity for visual effects (1.0 at beat, fades to 0)
		let intensity = 0;
		if (timingDiffMs <= 50) {
			intensity = 1.0;
		} else if (timingDiffMs <= 100) {
			intensity = 0.7;
		} else if (timingDiffMs <= this.settings.tolerance) {
			intensity = 0.4;
		}

		return {
			isOnBeat,
			beatProgress: beatPhase,
			timeToNext,
			intensity
		};
	}

	/**
	 * Check if currently on beat (for simple checks)
	 */
	isOnBeat(): boolean {
		const feedback = this.checkRhythm();
		return feedback.accuracy !== 'miss';
	}

	// ==================== Settings & State ====================

	updateSettings(settings: Partial<RhythmSettings>): void {
		const wasUsingMusic = this.settings.useMusic;
		
		// When using music, preserve the track's BPM (don't override from settings)
		const preserveBpm = this.musicLoaded && this.currentTrack && this.settings.useMusic;
		const trackBpm = this.currentTrack?.bpm;
		
		this.settings = { ...this.settings, ...settings };
		
		// Restore track BPM if using music
		if (preserveBpm && trackBpm) {
			this.settings.bpm = trackBpm;
			console.log(`🎵 Preserving track BPM: ${trackBpm} (ignoring settings)`);
		}
		
		// Update Transport BPM if running
		if (this.isRunning) {
			getTransport().bpm.value = this.settings.bpm;
		}
		
		// Handle music mode change
		if (settings.useMusic !== undefined && settings.useMusic !== wasUsingMusic) {
			if (settings.useMusic && !this.musicLoaded) {
				this.loadRandomTrack();
			}
		}
	}

	getSettings(): RhythmSettings {
		return { ...this.settings };
	}

	getCurrentTrack(): MusicTrack | null {
		return this.currentTrack;
	}

	/**
	 * Get the effective BPM being used
	 * When using music, returns the track's BPM
	 * When using metronome, returns the settings BPM
	 */
	getEffectiveBpm(): number {
		if (this.settings.useMusic && this.musicLoaded && this.currentTrack) {
			return this.currentTrack.bpm;
		}
		return this.settings.bpm;
	}
	
	/**
	 * @deprecated Use getEffectiveBpm() instead
	 */
	getDetectedBpm(): number {
		return this.getEffectiveBpm();
	}

	isUsingMusic(): boolean {
		return this.settings.useMusic === true && this.musicLoaded;
	}

	isActive(): boolean {
		return this.isRunning;
	}

	// ==================== Calibration ====================

	setCalibrationOffset(offsetMs: number): void {
		this.userCalibrationOffset = Math.max(-200, Math.min(200, offsetMs));
		console.log(`🎵 Calibration offset: ${this.userCalibrationOffset}ms`);
		
		// Disable auto-calibration if user manually sets offset
		if (offsetMs !== 0) {
			this.autoCalibrationEnabled = false;
			this.autoCalibrationSamples = [];
		}
	}

	getCalibrationOffset(): number {
		return this.userCalibrationOffset;
	}

	setAutoCalibration(enabled: boolean): void {
		this.autoCalibrationEnabled = enabled;
		if (!enabled) {
			this.autoCalibrationSamples = [];
		}
		console.log(`🎵 Auto-calibration ${enabled ? 'enabled' : 'disabled'}`);
	}

	isAutoCalibrationEnabled(): boolean {
		return this.autoCalibrationEnabled;
	}

	getAutoCalibrationSampleCount(): number {
		return this.autoCalibrationSamples.length;
	}

	onCalibrationChange(callback: ((offset: number) => void) | null): void {
		this.onCalibrationChangeCallback = callback;
	}

	resetAutoCalibration(): void {
		this.autoCalibrationSamples = [];
		this.userCalibrationOffset = 0;
		this.autoCalibrationEnabled = true;
		console.log('🎵 Auto-calibration reset');
		
		if (this.onCalibrationChangeCallback) {
			this.onCalibrationChangeCallback(0);
		}
	}

	private recordHitForAutoCalibration(signedTimingMs: number, accuracy: RhythmFeedback['accuracy']): void {
		if (!this.autoCalibrationEnabled) return;
		if (accuracy !== 'perfect' && accuracy !== 'good') return;
		
		this.autoCalibrationSamples.push(signedTimingMs);
		
		// Keep only recent samples
		if (this.autoCalibrationSamples.length > this.AUTO_CALIBRATION_MAX_SAMPLES) {
			this.autoCalibrationSamples.shift();
		}
		
		this.tryApplyAutoCalibration();
	}

	private tryApplyAutoCalibration(): void {
		if (this.autoCalibrationSamples.length < this.AUTO_CALIBRATION_MIN_SAMPLES) {
			return;
		}
		
		// Calculate median
		const sorted = [...this.autoCalibrationSamples].sort((a, b) => a - b);
		const mid = Math.floor(sorted.length / 2);
		const median = sorted.length % 2 === 0
			? (sorted[mid - 1] + sorted[mid]) / 2
			: sorted[mid];
		
		// Only apply if significant
		if (Math.abs(median) < 15) return;
		
		// Calculate adjustment
		const adjustment = -Math.round(median);
		const cappedAdjustment = Math.max(-this.AUTO_CALIBRATION_MAX_OFFSET, 
			Math.min(this.AUTO_CALIBRATION_MAX_OFFSET, adjustment));
		
		const newOffset = Math.max(-200, Math.min(200, this.userCalibrationOffset + cappedAdjustment));
		
		if (Math.abs(newOffset - this.userCalibrationOffset) >= 10) {
			const oldOffset = this.userCalibrationOffset;
			this.userCalibrationOffset = newOffset;
			
			console.log(`🎵 Auto-calibration: ${oldOffset}ms → ${newOffset}ms (median: ${median.toFixed(1)}ms)`);
			
			this.autoCalibrationSamples = [];
			
			if (this.onCalibrationChangeCallback) {
				this.onCalibrationChangeCallback(newOffset);
			}
		}
	}

	// ==================== Static Helpers ====================

	/**
	 * Parse rhythm settings from tournament description string
	 */
	static parseFromDescription(description: string): RhythmSettings | null {
		// Format: [RHYTHM_MODE:true,BPM:120,TOLERANCE:150,MUSIC:true]
		const rhythmMatch = description.match(/\[RHYTHM_MODE:true,BPM:(\d+),TOLERANCE:(\d+)(?:,MUSIC:(true|false))?\]/);
		if (!rhythmMatch) return null;

		return {
			enabled: true,
			bpm: parseInt(rhythmMatch[1], 10),
			tolerance: parseInt(rhythmMatch[2], 10),
			useMusic: rhythmMatch[3] === 'true'
		};
	}

	// ==================== Debug ====================

	getDebugInfo(): {
		isRunning: boolean;
		transportSeconds: number;
		transportPosition: string;
		bpm: number;
		beatPhase: number;
		currentBeat: number;
		calibrationOffset: number;
		autoCalibrationSamples: number;
	} {
		return {
			isRunning: this.isRunning,
			transportSeconds: getTransport().seconds,
			transportPosition: getTransport().position.toString(),
			bpm: this.settings.bpm,
			beatPhase: this.getBeatPhase(),
			currentBeat: this.currentBeat,
			calibrationOffset: this.userCalibrationOffset,
			autoCalibrationSamples: this.autoCalibrationSamples.length
		};
	}
}
