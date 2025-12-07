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

// Music tracks with their BPM (must match the actual music tempo)
export interface MusicTrack {
	name: string;
	url: string;
	bpm: number;
}

// Default music tracks - royalty-free game music
// These should be replaced with actual hosted music files
export const MUSIC_TRACKS: MusicTrack[] = [
	{ name: 'Energetic Beat', url: '/music/track1.mp3', bpm: 120 },
	{ name: 'Retro Groove', url: '/music/track2.mp3', bpm: 128 },
	{ name: 'Chill Vibes', url: '/music/track3.mp3', bpm: 100 },
	{ name: 'Fast Action', url: '/music/track4.mp3', bpm: 140 },
	{ name: 'Steady Pulse', url: '/music/track5.mp3', bpm: 110 },
];

export class RhythmEngine {
	private settings: RhythmSettings;
	private startTime: number = 0;
	private beatInterval: number = 0;
	private currentBeat: number = 0;
	private lastBeatTime: number = 0;
	private nextBeatTime: number = 0;
	private audioContext: AudioContext | null = null;
	private isRunning: boolean = false;
	private scheduledBeat: number = -1;
	private metronomeIntervalId: number | null = null;
	
	// Music playback
	private musicElement: HTMLAudioElement | null = null;
	private currentTrack: MusicTrack | null = null;
	private musicLoaded: boolean = false;

	constructor(settings: RhythmSettings) {
		this.settings = settings;
		this.beatInterval = (60 / settings.bpm) * 1000;
	}

	// Initialize audio context
	async initAudio(): Promise<void> {
		if (!this.audioContext) {
			this.audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
		}
		
		if (this.audioContext.state === 'suspended') {
			await this.audioContext.resume();
		}

		// If using music, preload a random track
		if (this.settings.useMusic) {
			await this.loadRandomTrack();
		}
	}

	// Load a random music track
	private async loadRandomTrack(): Promise<void> {
		// Pick random track
		const randomIndex = Math.floor(Math.random() * MUSIC_TRACKS.length);
		this.currentTrack = MUSIC_TRACKS[randomIndex];
		
		// Update BPM to match track
		this.settings.bpm = this.currentTrack.bpm;
		this.beatInterval = (60 / this.settings.bpm) * 1000;
		
		// Create audio element
		this.musicElement = new Audio(this.currentTrack.url);
		this.musicElement.loop = true;
		this.musicElement.volume = 0.5;
		
		// Preload
		try {
			this.musicElement.load();
			
			// Wait for canplaythrough event
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
				
				// Timeout after 5 seconds
				setTimeout(() => reject('Load timeout'), 5000);
			});
			
			this.musicLoaded = true;
			console.log(`🎵 Loaded track: ${this.currentTrack.name} (${this.currentTrack.bpm} BPM)`);
		} catch (error) {
			console.warn('🎵 Failed to load music, falling back to metronome:', error);
			this.settings.useMusic = false;
			this.musicLoaded = false;
		}
	}

	// Start the rhythm engine
	start(): void {
		if (!this.settings.enabled) return;
		
		this.startTime = performance.now();
		this.currentBeat = 0;
		this.lastBeatTime = this.startTime;
		this.nextBeatTime = this.startTime + this.beatInterval;
		this.isRunning = true;
		this.scheduledBeat = -1;
		
		if (this.settings.useMusic && this.musicElement && this.musicLoaded) {
			this.startMusic();
		} else {
			this.startMetronome();
		}
	}

	// Start music playback
	private startMusic(): void {
		if (!this.musicElement) return;
		
		this.musicElement.currentTime = 0;
		this.musicElement.play().catch(error => {
			console.warn('🎵 Music playback failed, falling back to metronome:', error);
			this.startMetronome();
		});
		
		console.log(`🎵 Playing: ${this.currentTrack?.name}`);
	}

	// Stop the rhythm engine
	stop(): void {
		this.isRunning = false;
		
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
	updateSettings(settings: Partial<RhythmSettings>): void {
		this.settings = { ...this.settings, ...settings };
		this.beatInterval = (60 / this.settings.bpm) * 1000;
		
		if (this.isRunning) {
			const elapsed = performance.now() - this.startTime;
			this.currentBeat = Math.floor(elapsed / this.beatInterval);
			this.lastBeatTime = this.startTime + (this.currentBeat * this.beatInterval);
			this.nextBeatTime = this.lastBeatTime + this.beatInterval;
		}
	}

	// Get current track info
	getCurrentTrack(): MusicTrack | null {
		return this.currentTrack;
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

		const elapsed = timestamp - this.startTime;
		this.currentBeat = Math.floor(elapsed / this.beatInterval);
		this.lastBeatTime = this.startTime + (this.currentBeat * this.beatInterval);
		this.nextBeatTime = this.lastBeatTime + this.beatInterval;

		const diffFromLastBeat = Math.abs(timestamp - this.lastBeatTime);
		const diffFromNextBeat = Math.abs(timestamp - this.nextBeatTime);
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
	getCurrentBeat(timestamp: number = performance.now()): { beat: number; progress: number; timeToNext: number } {
		if (!this.settings.enabled || !this.isRunning) {
			return { beat: 0, progress: 0, timeToNext: 0 };
		}

		const elapsed = timestamp - this.startTime;
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
			const elapsed = now - this.startTime;
			const currentBeatNum = Math.floor(elapsed / this.beatInterval);
			
			for (let i = 0; i <= 2; i++) {
				const beatNum = currentBeatNum + i;
				
				if (beatNum <= this.scheduledBeat) continue;
				
				const beatTimePerf = this.startTime + (beatNum * this.beatInterval);
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
}
