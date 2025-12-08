import { guess } from 'web-audio-beat-detector';

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

// Music tracks - BPM and offset will be auto-detected
export interface MusicTrack {
	name: string;
	url: string;
	// These are fallbacks if beat detection fails
	fallbackBpm: number;
	fallbackOffset: number;
}

// Music tracks from Crypt of the NecroDancer OST
// BPM values are from official sources: https://crypt-of-the-necrodancer.fandom.com/wiki/Music
export const MUSIC_TRACKS: MusicTrack[] = [
	{ name: 'Watch Your Step (Training)', url: '/music/track1.mp3', fallbackBpm: 120, fallbackOffset: 0 },
	{ name: 'Crypteque (1-2)', url: '/music/track2.mp3', fallbackBpm: 130, fallbackOffset: 0 },
	{ name: 'Tombtorial (Tutorial)', url: '/music/track3.mp3', fallbackBpm: 100, fallbackOffset: 0 },
];

export class RhythmEngine {
	private settings: RhythmSettings;
	private startTime: number = 0;
	private beatInterval: number = 0;
	private beatOffset: number = 0; // Offset from music start to first beat (in ms)
	private currentBeat: number = 0;
	private lastBeatTime: number = 0;
	private nextBeatTime: number = 0;
	private audioContext: AudioContext | null = null;
	private isRunning: boolean = false;
	private scheduledBeat: number = -1;
	private metronomeIntervalId: number | null = null;
	
	// Music playback
	private musicElement: HTMLAudioElement | null = null;
	private audioBuffer: AudioBuffer | null = null;
	private currentTrack: MusicTrack | null = null;
	private musicLoaded: boolean = false;
	private detectedBpm: number | null = null;
	private detectedOffset: number | null = null;
	
	// Phase 1 & 2: Audio synchronization tracking
	private isAudioPlaying: boolean = false;
	private audioStartTime: number = 0; // When audio actually started playing
	
	// Phase 3: User calibration offset (±200ms range)
	private userCalibrationOffset: number = 0;
	
	// Phase 4: Loop tracking
	private lastAudioTime: number = 0; // Track audio position for loop detection

	constructor(settings: RhythmSettings) {
		this.settings = settings;
		this.beatInterval = (60 / settings.bpm) * 1000;
	}
	
	// Phase 3: Set user calibration offset
	setCalibrationOffset(offsetMs: number): void {
		this.userCalibrationOffset = Math.max(-200, Math.min(200, offsetMs));
		console.log(`🎵 Calibration offset set to: ${this.userCalibrationOffset}ms`);
	}
	
	// Phase 3: Get current calibration offset
	getCalibrationOffset(): number {
		return this.userCalibrationOffset;
	}

	// Initialize audio context
	async initAudio(): Promise<void> {
		if (!this.audioContext) {
			this.audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
		}
		
		if (this.audioContext.state === 'suspended') {
			await this.audioContext.resume();
		}

		// If using music, preload a random track and detect its BPM
		if (this.settings.useMusic) {
			await this.loadRandomTrack();
		}
	}

	// Load a random music track and detect its BPM
	private async loadRandomTrack(): Promise<void> {
		// Pick random track
		const randomIndex = Math.floor(Math.random() * MUSIC_TRACKS.length);
		this.currentTrack = MUSIC_TRACKS[randomIndex];
		
		console.log(`🎵 Loading track: ${this.currentTrack.name}...`);
		
		try {
			// Fetch audio file as ArrayBuffer for beat detection
			const response = await fetch(this.currentTrack.url);
			if (!response.ok) throw new Error(`Failed to fetch: ${response.status}`);
			
			const arrayBuffer = await response.arrayBuffer();
			
			// Decode audio for beat detection
			if (!this.audioContext) {
				this.audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
			}
			
			this.audioBuffer = await this.audioContext.decodeAudioData(arrayBuffer.slice(0));
			
			// Use known BPM values for these tracks since web-audio-beat-detector can be unreliable
			// The fallback BPMs are manually verified to match the actual music
			console.log(`🎵 Loading track: "${this.currentTrack.name}"`);
			console.log(`🎵 Using known BPM: ${this.currentTrack.fallbackBpm}`);
			
			// Use the known/fallback BPM (these are accurate for our tracks)
			this.detectedBpm = this.currentTrack.fallbackBpm;
			
			// Try to detect the beat offset (first beat timing) which is more reliable
			try {
				const analysisLength = Math.min(30, this.audioBuffer.duration);
				const result = await guess(this.audioBuffer, 0, analysisLength);
				
				// Only use detected offset, not BPM (BPM detection is often wrong)
				this.detectedOffset = result.offset * 1000; // Convert to ms
				
				console.log(`🎵 Beat offset detected: ${this.detectedOffset.toFixed(0)}ms (detected BPM was ${result.bpm}, ignoring)`);
			} catch (detectError) {
				console.warn('🎵 Beat offset detection failed, using fallback:', detectError);
				this.detectedOffset = this.currentTrack.fallbackOffset;
			}
			
			// Update engine with detected values
			this.settings.bpm = this.detectedBpm!;
			this.beatInterval = (60 / this.settings.bpm) * 1000;
			this.beatOffset = this.detectedOffset!;
			
			console.log(`🎵 Final rhythm config: BPM=${this.settings.bpm}, interval=${this.beatInterval.toFixed(0)}ms, offset=${this.beatOffset.toFixed(0)}ms`);
			
			// Also create HTML Audio element for playback
			this.musicElement = new Audio(this.currentTrack.url);
			this.musicElement.loop = true;
			this.musicElement.volume = 0.5;
			
			// Phase 4: Add loop detection handler for re-sync
			this.musicElement.addEventListener('timeupdate', () => {
				if (!this.musicElement || !this.isAudioPlaying) return;
				
				const currentAudioTime = this.musicElement.currentTime * 1000;
				
				// Detect loop: audio time jumped backwards significantly
				if (this.lastAudioTime > 0 && currentAudioTime < this.lastAudioTime - 500) {
					// Music looped - recalculate beat based on new audio position
					const elapsed = currentAudioTime - this.beatOffset;
					if (elapsed >= 0) {
						this.currentBeat = Math.floor(elapsed / this.beatInterval);
						console.log(`🎵 Music looped, resynced to beat ${this.currentBeat} (audio: ${currentAudioTime.toFixed(0)}ms)`);
					}
				}
				
				this.lastAudioTime = currentAudioTime;
			});
			
			// Wait for it to be ready
			await new Promise<void>((resolve, reject) => {
				if (!this.musicElement) return reject('No music element');
				
				const onCanPlay = () => {
					this.musicElement?.removeEventListener('canplaythrough', onCanPlay);
					this.musicElement?.removeEventListener('error', onError);
					resolve();
				};
				
				const onError = () => {
					this.musicElement?.removeEventListener('canplaythrough', onCanPlay);
					this.musicElement?.removeEventListener('error', onError);
					reject('Failed to load audio');
				};
				
				this.musicElement.addEventListener('canplaythrough', onCanPlay);
				this.musicElement.addEventListener('error', onError);
				this.musicElement.load();
				
				// Timeout after 10 seconds
				setTimeout(() => reject('Load timeout'), 10000);
			});
			
			this.musicLoaded = true;
			console.log(`🎵 Ready: ${this.currentTrack.name} @ ${this.settings.bpm} BPM`);
			
		} catch (error) {
			console.warn('🎵 Failed to load music, falling back to metronome:', error);
			this.settings.useMusic = false;
			this.musicLoaded = false;
			
			// Reset to default BPM if we don't have music
			this.beatInterval = (60 / this.settings.bpm) * 1000;
			this.beatOffset = 0;
		}
	}

	// Start the rhythm engine
	start(): void {
		if (!this.settings.enabled) return;
		
		// For music: startTime is when we pressed play, but the first beat
		// happens at startTime + beatOffset.
		this.startTime = performance.now();
		this.currentBeat = 0;
		this.lastBeatTime = this.startTime + this.beatOffset;
		this.nextBeatTime = this.lastBeatTime + this.beatInterval;
		this.isRunning = true;
		this.scheduledBeat = -1;
		
		if (this.settings.useMusic && this.musicElement && this.musicLoaded) {
			this.startMusic();
		} else {
			this.startMetronome();
		}
	}

	// Start music playback
	// Phase 1: Capture actual audio start time for accurate sync
	private startMusic(): void {
		if (!this.musicElement) return;
		
		this.musicElement.currentTime = 0;
		this.lastAudioTime = 0; // Reset loop tracking
		this.isAudioPlaying = false;
		
		// Phase 1: Listen for actual playback start to sync timing
		const onPlaying = () => {
			this.audioStartTime = performance.now();
			this.isAudioPlaying = true;
			
			// Re-sync beat timing with actual audio start (not when play() was called)
			this.startTime = this.audioStartTime;
			this.lastBeatTime = this.startTime + this.beatOffset;
			this.nextBeatTime = this.lastBeatTime + this.beatInterval;
			this.currentBeat = 0;
			
			console.log(`🎵 Audio started at: ${this.audioStartTime.toFixed(0)}ms (latency compensated)`);
			
			this.musicElement?.removeEventListener('playing', onPlaying);
		};
		
		this.musicElement.addEventListener('playing', onPlaying);
		
		this.musicElement.play().catch(error => {
			console.warn('🎵 Music playback failed, falling back to metronome:', error);
			this.musicElement?.removeEventListener('playing', onPlaying);
			this.startMetronome();
		});
		
		console.log(`🎵 Starting: ${this.currentTrack?.name} (${this.settings.bpm} BPM, offset: ${this.beatOffset.toFixed(0)}ms)`);
	}

	// Stop the rhythm engine
	stop(): void {
		this.isRunning = false;
		this.isAudioPlaying = false;
		this.lastAudioTime = 0;
		
		// Stop metronome
		if (this.metronomeIntervalId !== null) {
			clearInterval(this.metronomeIntervalId);
			this.metronomeIntervalId = null;
		}
		
		// Stop music
		if (this.musicElement) {
			this.musicElement.pause();
			this.musicElement.currentTime = 0;
		}
		
		// Close audio context
		if (this.audioContext) {
			this.audioContext.close();
			this.audioContext = null;
		}
	}

	// Update rhythm settings
	// NOTE: When music is loaded, we preserve the detected BPM and offset
	// The tournament BPM is only used as initial/fallback value
	updateSettings(settings: Partial<RhythmSettings>): void {
		// Preserve detected BPM and offset if music is loaded
		const preserveBpm = this.musicLoaded && this.detectedBpm !== null;
		const preserveOffset = this.musicLoaded && this.detectedOffset !== null;
		
		this.settings = { ...this.settings, ...settings };
		
		// Restore detected values if music is loaded (they are the source of truth)
		if (preserveBpm) {
			this.settings.bpm = this.detectedBpm!;
			console.log(`🎵 Preserving detected BPM: ${this.detectedBpm} (ignoring settings BPM: ${settings.bpm})`);
		}
		if (preserveOffset) {
			this.beatOffset = this.detectedOffset!;
		}
		
		this.beatInterval = (60 / this.settings.bpm) * 1000;
		
		if (this.isRunning) {
			const elapsed = performance.now() - this.startTime - this.beatOffset;
			if (elapsed >= 0) {
				this.currentBeat = Math.floor(elapsed / this.beatInterval);
				this.lastBeatTime = this.startTime + this.beatOffset + (this.currentBeat * this.beatInterval);
				this.nextBeatTime = this.lastBeatTime + this.beatInterval;
			}
		}
	}

	// Get current track info
	getCurrentTrack(): MusicTrack | null {
		return this.currentTrack;
	}

	// Get detected BPM
	getDetectedBpm(): number | null {
		return this.detectedBpm;
	}

	// Get detected offset
	getDetectedOffset(): number | null {
		return this.detectedOffset;
	}

	// Check if using music
	isUsingMusic(): boolean {
		return this.settings.useMusic === true && this.musicLoaded;
	}

	// Check if a move is on rhythm and provide feedback
	checkRhythm(timestamp: number = performance.now()): RhythmFeedback {
		if (!this.settings.enabled || !this.isRunning) {
			return {
				accuracy: 'perfect',
				timingDiff: 0,
				beatNumber: 0,
				score: 0
			};
		}

		// Phase 2: Use audio time as source of truth when music is playing
		// Phase 3: Apply user calibration offset
		let elapsed: number;
		if (this.settings.useMusic && this.musicElement && this.isAudioPlaying) {
			// Use actual audio position - this is THE source of truth for music mode
			const audioTimeMs = this.musicElement.currentTime * 1000;
			elapsed = audioTimeMs - this.beatOffset - this.userCalibrationOffset;
		} else {
			// Metronome mode: use performance.now() based timing
			elapsed = timestamp - this.startTime - this.beatOffset - this.userCalibrationOffset;
		}
		
		// Before the first beat
		if (elapsed < 0) {
			const timeToFirstBeat = -elapsed;
			return {
				accuracy: timeToFirstBeat <= this.settings.tolerance ? 'early' : 'miss',
				timingDiff: timeToFirstBeat,
				beatNumber: 0,
				score: timeToFirstBeat <= this.settings.tolerance ? 25 : 0
			};
		}
		
		this.currentBeat = Math.floor(elapsed / this.beatInterval);
		
		// Calculate beat times based on elapsed time (not startTime for music mode)
		const beatStartTime = this.currentBeat * this.beatInterval;
		const beatEndTime = (this.currentBeat + 1) * this.beatInterval;
		
		const diffFromLastBeat = Math.abs(elapsed - beatStartTime);
		const diffFromNextBeat = Math.abs(elapsed - beatEndTime);
		const timingDiff = Math.min(diffFromLastBeat, diffFromNextBeat);

		let accuracy: RhythmFeedback['accuracy'];
		let score: number;

		if (timingDiff <= 50) {
			accuracy = 'perfect';
			score = 100;
		} else if (timingDiff <= 100) {
			accuracy = 'good';
			score = 50;
		} else if (timingDiff <= this.settings.tolerance) {
			accuracy = diffFromLastBeat < diffFromNextBeat ? 'late' : 'early';
			score = 25;
		} else {
			accuracy = 'miss';
			score = 0;
		}

		return {
			accuracy,
			timingDiff,
			beatNumber: this.currentBeat,
			score
		};
	}

	// Get current beat info
	// Phase 2: Use audio time as source of truth for visual feedback
	// Phase 3: Apply user calibration offset
	getCurrentBeat(timestamp: number = performance.now()): { beat: number; progress: number; timeToNext: number } {
		if (!this.settings.enabled || !this.isRunning) {
			return { beat: 0, progress: 0, timeToNext: 0 };
		}

		// Phase 2: Use audio time as source of truth when music is playing
		let elapsed: number;
		if (this.settings.useMusic && this.musicElement && this.isAudioPlaying) {
			// Use actual audio position for visual sync
			const audioTimeMs = this.musicElement.currentTime * 1000;
			elapsed = audioTimeMs - this.beatOffset - this.userCalibrationOffset;
		} else {
			// Metronome mode: use performance.now() based timing
			elapsed = timestamp - this.startTime - this.beatOffset - this.userCalibrationOffset;
		}
		
		// Before first beat
		if (elapsed < 0) {
			const timeToFirstBeat = -elapsed;
			const twoBeatDuration = this.beatInterval * 2;
			const progress = timeToFirstBeat < twoBeatDuration 
				? 1 - (timeToFirstBeat / twoBeatDuration)
				: 0;
			return { beat: -1, progress, timeToNext: timeToFirstBeat };
		}
		
		const beat = Math.floor(elapsed / this.beatInterval);
		const progress = (elapsed % this.beatInterval) / this.beatInterval;
		const timeToNext = this.beatInterval - (elapsed % this.beatInterval);

		return { beat, progress, timeToNext };
	}

	// Check if currently on beat
	isOnBeat(timestamp: number = performance.now()): boolean {
		const feedback = this.checkRhythm(timestamp);
		return feedback.accuracy !== 'miss';
	}

	// Play metronome tick at specific audio time
	private playTickAtTime(audioTime: number, frequency: number = 800, duration: number = 0.03): void {
		if (!this.audioContext) return;

		const oscillator = this.audioContext.createOscillator();
		const gainNode = this.audioContext.createGain();

		oscillator.connect(gainNode);
		gainNode.connect(this.audioContext.destination);

		oscillator.frequency.value = frequency;
		oscillator.type = 'sine';

		gainNode.gain.setValueAtTime(0.3, audioTime);
		gainNode.gain.exponentialRampToValueAtTime(0.01, audioTime + duration);

		oscillator.start(audioTime);
		oscillator.stop(audioTime + duration);
	}

	// Start metronome with precise scheduling
	private startMetronome(): void {
		if (!this.settings.enabled || !this.isRunning || !this.audioContext) return;

		const scheduleAhead = 0.1;
		const checkInterval = 25;
		
		const scheduler = () => {
			if (!this.isRunning || !this.audioContext) return;

			const now = performance.now();
			const audioNow = this.audioContext.currentTime;
			const elapsed = now - this.startTime - this.beatOffset;
			
			// Before first beat, check if we need to schedule beat 0
			if (elapsed < 0) {
				const timeToFirstBeat = -elapsed;
				if (timeToFirstBeat <= scheduleAhead * 1000 && this.scheduledBeat < 0) {
					const beatTimeAudio = audioNow + (timeToFirstBeat / 1000);
					this.playTickAtTime(beatTimeAudio, 1000, 0.03);
					this.scheduledBeat = 0;
				}
				return;
			}
			
			const currentBeatNum = Math.floor(elapsed / this.beatInterval);
			
			for (let i = 0; i <= 2; i++) {
				const beatNum = currentBeatNum + i;
				
				if (beatNum <= this.scheduledBeat) continue;
				
				const beatTimePerf = this.startTime + this.beatOffset + (beatNum * this.beatInterval);
				const msUntilBeat = beatTimePerf - now;
				
				if (msUntilBeat < 0 || msUntilBeat > scheduleAhead * 1000) continue;
				
				const beatTimeAudio = audioNow + (msUntilBeat / 1000);
				const frequency = (beatNum % 4 === 0) ? 1000 : 800;
				this.playTickAtTime(beatTimeAudio, frequency, 0.03);
				
				this.scheduledBeat = beatNum;
			}
		};

		this.metronomeIntervalId = window.setInterval(scheduler, checkInterval);
		scheduler();
	}

	// Parse rhythm settings from description string
	static parseFromDescription(description: string): RhythmSettings | null {
		// Extended format: [RHYTHM_MODE:true,BPM:120,TOLERANCE:150,MUSIC:true]
		const rhythmMatch = description.match(/\[RHYTHM_MODE:true,BPM:(\d+),TOLERANCE:(\d+)(?:,MUSIC:(true|false))?\]/);
		if (!rhythmMatch) return null;

		return {
			enabled: true,
			bpm: parseInt(rhythmMatch[1], 10),
			tolerance: parseInt(rhythmMatch[2], 10),
			useMusic: rhythmMatch[3] === 'true'
		};
	}

	// Get visual feedback data for UI
	getVisualFeedback(timestamp: number = performance.now()): {
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

		const { progress, timeToNext } = this.getCurrentBeat(timestamp);
		const feedback = this.checkRhythm(timestamp);
		
		const maxIntensity = 1;
		let intensity = 0;
		
		if (feedback.timingDiff <= 50) {
			intensity = maxIntensity;
		} else if (feedback.timingDiff <= 100) {
			intensity = 0.7;
		} else if (feedback.timingDiff <= this.settings.tolerance) {
			intensity = 0.4;
		}

		return {
			isOnBeat: feedback.accuracy !== 'miss',
			beatProgress: progress,
			timeToNext,
			intensity
		};
	}
	
	// Debug: Get sync status for verification
	getDebugInfo(): {
		isRunning: boolean;
		isAudioPlaying: boolean;
		audioTimeMs: number;
		performanceNow: number;
		startTime: number;
		audioStartTime: number;
		beatOffset: number;
		calibrationOffset: number;
		currentBeat: number;
		beatInterval: number;
		drift: number; // Difference between audio time and calculated time
	} {
		const now = performance.now();
		const audioTimeMs = this.musicElement?.currentTime ? this.musicElement.currentTime * 1000 : 0;
		const expectedAudioTime = now - this.audioStartTime;
		const drift = this.isAudioPlaying ? audioTimeMs - expectedAudioTime : 0;
		
		return {
			isRunning: this.isRunning,
			isAudioPlaying: this.isAudioPlaying,
			audioTimeMs,
			performanceNow: now,
			startTime: this.startTime,
			audioStartTime: this.audioStartTime,
			beatOffset: this.beatOffset,
			calibrationOffset: this.userCalibrationOffset,
			currentBeat: this.currentBeat,
			beatInterval: this.beatInterval,
			drift
		};
	}
}
