<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { RhythmEngine } from '$lib/game/rhythmEngine.js';

	export let rhythmEngine: RhythmEngine;
	export let showCalibration: boolean = false; // Toggle calibration UI

	let beatProgress = 0;
	let isOnBeat = false;
	let animationFrame: number;
	let showMissFlash = false;
	let heartScale = 1;
	let heartBeat = false;
	
	// Track bars moving from edges to center
	let leftBarPos = 0;
	let rightBarPos = 100;
	let barApproaching = false;
	
	// Calibration state
	let calibrationOffset = 0;
	let showCalibrationPanel = false;
	let autoCalibrationEnabled = true;
	let autoCalibrationSamples = 0;

	// Animation loop
	const animate = () => {
		if (rhythmEngine) {
			const visual = rhythmEngine.getVisualFeedback();
			beatProgress = visual.beatProgress;
			isOnBeat = visual.isOnBeat;
			
			// The beat timing works like this:
			// - progress=0: start of beat interval (beat just happened)
			// - progress=0.5: middle of beat interval
			// - progress=1: end of beat interval (about to hit next beat)
			// 
			// The tolerance window in checkRhythm() allows hits near BOTH edges:
			// - progress near 0 = close to last beat (late hit)
			// - progress near 1 = close to next beat (early hit)
			//
			// For the visual, we want bars to converge at center when you should hit.
			// We'll use a "ping-pong" style where bars reach center at both progress≈0 and progress≈1
			//
			// Map progress to position:
			// - progress 0→0.5: bars move from center (50) to edge (0/100)
			// - progress 0.5→1: bars move from edge (0/100) back to center (50)
			//
			// This creates a "bounce" effect where bars are at center at beat boundaries
			
			let normalizedProgress: number;
			if (beatProgress <= 0.5) {
				// First half: center to edge (50 → 0)
				normalizedProgress = beatProgress * 2; // 0→1
				leftBarPos = 50 * (1 - normalizedProgress); // 50→0
				rightBarPos = 50 + 50 * normalizedProgress; // 50→100
			} else {
				// Second half: edge to center (0 → 50)
				normalizedProgress = (beatProgress - 0.5) * 2; // 0→1
				leftBarPos = 50 * normalizedProgress; // 0→50
				rightBarPos = 100 - 50 * normalizedProgress; // 100→50
			}
			
			// Bar is "approaching" (green) when close to center
			barApproaching = beatProgress < 0.15 || beatProgress > 0.85;
			
			// Heart pulse effect - pulse when bars are at/near center
			if (beatProgress < 0.1 || beatProgress > 0.9) {
				heartScale = 1.5;
				heartBeat = true;
			} else if (beatProgress < 0.2 || beatProgress > 0.8) {
				heartScale = 1.2;
				heartBeat = true;
			} else {
				heartScale = 1;
				heartBeat = false;
			}
		}
		animationFrame = requestAnimationFrame(animate);
	};

	// Show miss flash (called externally when move blocked)
	export function showMiss() {
		showMissFlash = true;
		setTimeout(() => {
			showMissFlash = false;
		}, 200);
	}
	
	// Calibration functions
	const handleCalibrationChange = (e: Event) => {
		const target = e.target as HTMLInputElement;
		calibrationOffset = parseInt(target.value, 10);
		rhythmEngine?.setCalibrationOffset(calibrationOffset);
	};
	
	const toggleCalibrationPanel = () => {
		showCalibrationPanel = !showCalibrationPanel;
	};
	
	const resetCalibration = () => {
		calibrationOffset = 0;
		rhythmEngine?.resetAutoCalibration();
		autoCalibrationEnabled = true;
	};
	
	const toggleAutoCalibration = () => {
		autoCalibrationEnabled = !autoCalibrationEnabled;
		rhythmEngine?.setAutoCalibration(autoCalibrationEnabled);
	};
	
	// Update auto-calibration sample count periodically
	const updateAutoCalibrationStatus = () => {
		if (rhythmEngine) {
			autoCalibrationSamples = rhythmEngine.getAutoCalibrationSampleCount();
			autoCalibrationEnabled = rhythmEngine.isAutoCalibrationEnabled();
		}
	};

	let statusInterval: ReturnType<typeof setInterval>;
	
	onMount(() => {
		// Initialize calibration offset from engine
		calibrationOffset = rhythmEngine?.getCalibrationOffset() ?? 0;
		autoCalibrationEnabled = rhythmEngine?.isAutoCalibrationEnabled() ?? true;
		
		// Set up callback for when auto-calibration changes the offset
		rhythmEngine?.onCalibrationChange((newOffset: number) => {
			calibrationOffset = newOffset;
		});
		
		animate();
		
		// Update auto-calibration status periodically
		statusInterval = setInterval(updateAutoCalibrationStatus, 500);
	});

	onDestroy(() => {
		if (animationFrame) {
			cancelAnimationFrame(animationFrame);
		}
		if (statusInterval) {
			clearInterval(statusInterval);
		}
		// Clear callback to prevent memory leaks
		rhythmEngine?.onCalibrationChange(null);
	});
</script>

<div class="beat-indicator" class:miss-flash={showMissFlash}>
	<!-- Track background -->
	<div class="beat-track">
		<!-- Left bar moving right -->
		<div 
			class="beat-bar left"
			class:approaching={barApproaching}
			style="left: {leftBarPos}%;"
		></div>

		<!-- Center heart/target zone -->
		<div class="beat-center" class:on-beat={isOnBeat}>
			<div 
				class="heart"
				class:beating={heartBeat}
				style="transform: scale({heartScale});"
			>
				<svg viewBox="0 0 24 24" fill="currentColor">
					<path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
				</svg>
			</div>
		</div>

		<!-- Right bar moving left -->
		<div 
			class="beat-bar right"
			class:approaching={barApproaching}
			style="left: {rightBarPos}%;"
		></div>
	</div>
	
	<!-- Calibration toggle button -->
	{#if showCalibration}
		<button 
			class="calibration-toggle"
			onclick={toggleCalibrationPanel}
			title="Adjust beat timing"
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<circle cx="12" cy="12" r="3"/>
				<path d="M12 1v6m0 6v10M4.22 4.22l4.24 4.24m7.08 7.08l4.24 4.24M1 12h6m6 0h10M4.22 19.78l4.24-4.24m7.08-7.08l4.24-4.24"/>
			</svg>
		</button>
	{/if}
	
	<!-- Calibration panel -->
	{#if showCalibrationPanel}
		<div class="calibration-panel">
			<div class="calibration-header">
				<span class="calibration-title">Beat Calibration</span>
				<button class="calibration-close" onclick={toggleCalibrationPanel}>×</button>
			</div>
			<div class="calibration-content">
				<!-- Auto-calibration status -->
				<div class="auto-calibration-section">
					<button 
						class="auto-calibration-toggle"
						class:enabled={autoCalibrationEnabled}
						onclick={toggleAutoCalibration}
					>
						<span class="auto-dot" class:active={autoCalibrationEnabled}></span>
						<span>Auto-Calibrate</span>
					</button>
					{#if autoCalibrationEnabled}
						<span class="auto-status">
							{#if autoCalibrationSamples < 10}
								Learning... ({autoCalibrationSamples}/10)
							{:else}
								Active ({autoCalibrationSamples} samples)
							{/if}
						</span>
					{:else}
						<span class="auto-status disabled">Disabled</span>
					{/if}
				</div>
				
				<div class="calibration-divider"></div>
				
				<p class="calibration-hint">
					Manual: Slide right (+) if visuals feel early, left (-) if late.
				</p>
				<div class="calibration-slider-container">
					<span class="calibration-label">-200ms</span>
					<input 
						type="range" 
						min="-200" 
						max="200" 
						value={calibrationOffset}
						oninput={handleCalibrationChange}
						class="calibration-slider"
					/>
					<span class="calibration-label">+200ms</span>
				</div>
				<div class="calibration-value">
					<span>Offset: <strong>{calibrationOffset}ms</strong></span>
					<button class="calibration-reset" onclick={resetCalibration}>Reset</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.beat-indicator {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
		padding: 0.5rem;
		transition: all 0.1s ease;
		width: 100%;
	}

	.beat-indicator.miss-flash {
		animation: missShake 0.2s ease-out;
	}

	@keyframes missShake {
		0%, 100% { transform: translateX(0); }
		25% { transform: translateX(-5px); }
		75% { transform: translateX(5px); }
	}

	.beat-track {
		position: relative;
		width: 100%;
		height: 56px;
		background: linear-gradient(90deg, 
			rgba(20, 20, 30, 0.95) 0%,
			rgba(30, 30, 45, 0.95) 40%,
			rgba(34, 197, 94, 0.25) 47%,
			rgba(34, 197, 94, 0.25) 53%,
			rgba(30, 30, 45, 0.95) 60%,
			rgba(20, 20, 30, 0.95) 100%
		);
		border-radius: 8px;
		overflow: hidden;
		border: 2px solid rgba(139, 92, 246, 0.4);
		box-shadow: inset 0 0 30px rgba(0, 0, 0, 0.6);
	}

	/* Beat bars */
	.beat-bar {
		position: absolute;
		top: 8px;
		bottom: 8px;
		width: 8px;
		background: linear-gradient(180deg, 
			rgba(139, 92, 246, 0.8) 0%,
			rgba(168, 85, 247, 1) 50%,
			rgba(139, 92, 246, 0.8) 100%
		);
		border-radius: 4px;
		transform: translateX(-50%);
		box-shadow: 0 0 12px rgba(139, 92, 246, 0.7),
		            0 0 4px rgba(139, 92, 246, 1);
	}

	.beat-bar.approaching {
		background: linear-gradient(180deg, 
			rgba(34, 197, 94, 0.9) 0%,
			rgba(74, 222, 128, 1) 50%,
			rgba(34, 197, 94, 0.9) 100%
		);
		box-shadow: 0 0 20px rgba(34, 197, 94, 0.9),
		            0 0 6px rgba(34, 197, 94, 1);
	}

	/* Center target zone */
	.beat-center {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 56px;
		height: 56px;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 10;
	}

	.heart {
		width: 36px;
		height: 36px;
		color: #4b5563;
		transition: transform 0.06s ease-out, color 0.08s ease;
		filter: drop-shadow(0 0 4px rgba(75, 85, 99, 0.3));
	}

	.heart.beating {
		color: #ef4444;
		filter: drop-shadow(0 0 16px rgba(239, 68, 68, 0.9));
	}

	.beat-center.on-beat .heart {
		color: #22c55e;
		filter: drop-shadow(0 0 20px rgba(34, 197, 94, 1));
	}

	/* Calibration toggle button */
	.calibration-toggle {
		position: absolute;
		top: 4px;
		right: 4px;
		width: 24px;
		height: 24px;
		padding: 4px;
		background: rgba(139, 92, 246, 0.3);
		border: 1px solid rgba(139, 92, 246, 0.5);
		border-radius: 4px;
		color: rgba(255, 255, 255, 0.7);
		cursor: pointer;
		transition: all 0.2s ease;
		z-index: 20;
	}
	
	.calibration-toggle:hover {
		background: rgba(139, 92, 246, 0.5);
		color: white;
	}
	
	.calibration-toggle svg {
		width: 100%;
		height: 100%;
	}
	
	/* Calibration panel */
	.calibration-panel {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		margin-top: 8px;
		background: rgba(20, 20, 30, 0.95);
		border: 1px solid rgba(139, 92, 246, 0.4);
		border-radius: 8px;
		padding: 12px;
		z-index: 30;
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.5);
	}
	
	.calibration-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 8px;
	}
	
	.calibration-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: #a78bfa;
	}
	
	.calibration-close {
		width: 20px;
		height: 20px;
		background: transparent;
		border: none;
		color: #9ca3af;
		font-size: 1.25rem;
		cursor: pointer;
		line-height: 1;
	}
	
	.calibration-close:hover {
		color: white;
	}
	
	.calibration-content {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	
	/* Auto-calibration section */
	.auto-calibration-section {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	
	.auto-calibration-toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px;
		background: rgba(34, 197, 94, 0.1);
		border: 1px solid rgba(34, 197, 94, 0.3);
		border-radius: 4px;
		color: #9ca3af;
		font-size: 0.75rem;
		cursor: pointer;
		transition: all 0.2s ease;
	}
	
	.auto-calibration-toggle:hover {
		background: rgba(34, 197, 94, 0.2);
	}
	
	.auto-calibration-toggle.enabled {
		background: rgba(34, 197, 94, 0.2);
		border-color: rgba(34, 197, 94, 0.5);
		color: #4ade80;
	}
	
	.auto-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: #6b7280;
		transition: all 0.2s ease;
	}
	
	.auto-dot.active {
		background: #4ade80;
		box-shadow: 0 0 8px rgba(74, 222, 128, 0.6);
		animation: pulse-dot 2s ease-in-out infinite;
	}
	
	@keyframes pulse-dot {
		0%, 100% { opacity: 1; }
		50% { opacity: 0.5; }
	}
	
	.auto-status {
		font-size: 0.625rem;
		color: #4ade80;
	}
	
	.auto-status.disabled {
		color: #6b7280;
	}
	
	.calibration-divider {
		height: 1px;
		background: rgba(139, 92, 246, 0.2);
		margin: 4px 0;
	}
	
	.calibration-hint {
		font-size: 0.75rem;
		color: #9ca3af;
		margin: 0;
		line-height: 1.4;
	}
	
	.calibration-slider-container {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	
	.calibration-label {
		font-size: 0.625rem;
		color: #6b7280;
		white-space: nowrap;
	}
	
	.calibration-slider {
		flex: 1;
		-webkit-appearance: none;
		appearance: none;
		height: 6px;
		background: #374151;
		border-radius: 3px;
		outline: none;
	}
	
	.calibration-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 16px;
		height: 16px;
		background: #a78bfa;
		border-radius: 50%;
		cursor: pointer;
		transition: background 0.2s ease;
	}
	
	.calibration-slider::-webkit-slider-thumb:hover {
		background: #c4b5fd;
	}
	
	.calibration-slider::-moz-range-thumb {
		width: 16px;
		height: 16px;
		background: #a78bfa;
		border-radius: 50%;
		cursor: pointer;
		border: none;
	}
	
	.calibration-value {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 0.75rem;
		color: #d1d5db;
	}
	
	.calibration-value strong {
		color: #22c55e;
	}
	
	.calibration-reset {
		padding: 2px 8px;
		background: rgba(239, 68, 68, 0.2);
		border: 1px solid rgba(239, 68, 68, 0.4);
		border-radius: 4px;
		color: #f87171;
		font-size: 0.625rem;
		cursor: pointer;
		transition: all 0.2s ease;
	}
	
	.calibration-reset:hover {
		background: rgba(239, 68, 68, 0.3);
		color: #fca5a5;
	}

	/* Responsive */
	@media (max-width: 640px) {
		.beat-track {
			height: 48px;
		}

		.beat-bar {
			width: 6px;
			top: 6px;
			bottom: 6px;
		}

		.beat-center {
			width: 48px;
			height: 48px;
		}

		.heart {
			width: 30px;
			height: 30px;
		}
		
		.calibration-toggle {
			width: 20px;
			height: 20px;
			padding: 3px;
		}
		
		.calibration-panel {
			padding: 10px;
		}
		
		.calibration-title {
			font-size: 0.75rem;
		}
		
		.calibration-hint {
			font-size: 0.625rem;
		}
	}
</style>
