export interface RhythmSettings {
	enabled: boolean;
	bpm: number;
	tolerance: number; // milliseconds
}

export interface RhythmFeedback {
	accuracy: 'perfect' | 'good' | 'miss' | 'early' | 'late';
	timingDiff: number; // milliseconds from beat
	beatNumber: number;
	score: number;
}

export class RhythmEngine {
	private settings: RhythmSettings;
	private startTime: number = 0;
	private beatInterval: number = 0;
	private currentBeat: number = 0;
	private lastBeatTime: number = 0;
	private nextBeatTime: number = 0;
	private audioContext: AudioContext | null = null;
	private isRunning: boolean = false;
	private scheduledBeat: number = -1; // Track which beat we've scheduled sound for
	private metronomeIntervalId: number | null = null;

	constructor(settings: RhythmSettings) {
		this.settings = settings;
		this.beatInterval = (60 / settings.bpm) * 1000; // Convert BPM to milliseconds
	}

	// Initialize audio context for metronome
	async initAudio(): Promise<void> {
		if (!this.audioContext) {
			this.audioContext = new (window.AudioContext || (window as any).webkitAudioContext)();
		}
		
		// Resume audio context if suspended (browser policy)
		if (this.audioContext.state === 'suspended') {
			await this.audioContext.resume();
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
		this.scheduledBeat = -1; // Reset scheduled beat tracking
		
		// Start metronome if audio is enabled
		this.startMetronome();
	}

	// Stop the rhythm engine
	stop(): void {
		this.isRunning = false;
		if (this.metronomeIntervalId !== null) {
			clearInterval(this.metronomeIntervalId);
			this.metronomeIntervalId = null;
		}
		if (this.audioContext) {
			this.audioContext.close();
			this.audioContext = null;
		}
	}

	// Update rhythm settings
	updateSettings(settings: Partial<RhythmSettings>): void {
		this.settings = { ...this.settings, ...settings };
		this.beatInterval = (60 / this.settings.bpm) * 1000;
		
		// Recalculate next beat time if running
		if (this.isRunning) {
			const elapsed = performance.now() - this.startTime;
			this.currentBeat = Math.floor(elapsed / this.beatInterval);
			this.lastBeatTime = this.startTime + (this.currentBeat * this.beatInterval);
			this.nextBeatTime = this.lastBeatTime + this.beatInterval;
		}
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

		// Update current beat based on timestamp
		const elapsed = timestamp - this.startTime;
		this.currentBeat = Math.floor(elapsed / this.beatInterval);
		this.lastBeatTime = this.startTime + (this.currentBeat * this.beatInterval);
		this.nextBeatTime = this.lastBeatTime + this.beatInterval;

		// Calculate timing difference from nearest beat
		const diffFromLastBeat = Math.abs(timestamp - this.lastBeatTime);
		const diffFromNextBeat = Math.abs(timestamp - this.nextBeatTime);
		const timingDiff = Math.min(diffFromLastBeat, diffFromNextBeat);

		// Determine accuracy and score
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

	// Check if currently on beat (within tolerance)
	isOnBeat(timestamp: number = performance.now()): boolean {
		const feedback = this.checkRhythm(timestamp);
		return feedback.accuracy !== 'miss';
	}

	// Play metronome tick sound at a specific audio time
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

	// Start metronome ticks with precise audio scheduling
	private startMetronome(): void {
		if (!this.settings.enabled || !this.isRunning || !this.audioContext) return;

		// Schedule beats ahead of time for precise timing
		const scheduleAhead = 0.1; // Schedule 100ms ahead
		const checkInterval = 25; // Check every 25ms
		
		const scheduler = () => {
			if (!this.isRunning || !this.audioContext) return;

			const now = performance.now();
			const audioNow = this.audioContext.currentTime;
			
			// Calculate which beat we're on and when the next beats occur
			const elapsed = now - this.startTime;
			const currentBeatNum = Math.floor(elapsed / this.beatInterval);
			
			// Schedule upcoming beats
			for (let i = 0; i <= 2; i++) {
				const beatNum = currentBeatNum + i;
				
				// Skip if we've already scheduled this beat
				if (beatNum <= this.scheduledBeat) continue;
				
				// Calculate when this beat should play (in performance.now time)
				const beatTimePerf = this.startTime + (beatNum * this.beatInterval);
				const msUntilBeat = beatTimePerf - now;
				
				// Only schedule if it's within our lookahead window
				if (msUntilBeat < 0 || msUntilBeat > scheduleAhead * 1000) continue;
				
				// Convert to audio context time
				const beatTimeAudio = audioNow + (msUntilBeat / 1000);
				
				// Different frequency for downbeat (every 4 beats)
				const frequency = (beatNum % 4 === 0) ? 1000 : 800;
				this.playTickAtTime(beatTimeAudio, frequency, 0.03);
				
				this.scheduledBeat = beatNum;
			}
		};

		// Run scheduler at regular intervals
		this.metronomeIntervalId = window.setInterval(scheduler, checkInterval);
		
		// Initial schedule
		scheduler();
	}

	// Parse rhythm settings from description string
	static parseFromDescription(description: string): RhythmSettings | null {
		const rhythmMatch = description.match(/\[RHYTHM_MODE:true,BPM:(\d+),TOLERANCE:(\d+)\]/);
		if (!rhythmMatch) return null;

		return {
			enabled: true,
			bpm: parseInt(rhythmMatch[1], 10),
			tolerance: parseInt(rhythmMatch[2], 10)
		};
	}

	// Get visual feedback data for UI
	getVisualFeedback(timestamp: number = performance.now()): {
		isOnBeat: boolean;
		beatProgress: number;
		timeToNext: number;
		intensity: number; // 0-1, how close to beat
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
		
		// Calculate intensity based on timing difference (same logic as checkRhythm)
		// This ensures visual feedback matches actual move validation
		const maxIntensity = 1;
		let intensity = 0;
		
		if (feedback.timingDiff <= 50) {
			intensity = maxIntensity; // Perfect
		} else if (feedback.timingDiff <= 100) {
			intensity = 0.7; // Good
		} else if (feedback.timingDiff <= this.settings.tolerance) {
			intensity = 0.4; // Early/Late but valid
		}

		return {
			isOnBeat: feedback.accuracy !== 'miss',
			beatProgress: progress,
			timeToNext,
			intensity
		};
	}
}