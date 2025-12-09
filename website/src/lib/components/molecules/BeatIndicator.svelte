<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { RhythmEngine } from '$lib/game/rhythmEngine.js';

	export let rhythmEngine: RhythmEngine;

	// Visual state - updated every frame
	let beatPhase = 0;
	let intensity = 0;
	let isInWindow = false;
	
	// Bar positions (derived from beatPhase)
	let leftBarPos = 50;
	let rightBarPos = 50;
	
	// Animation
	let animationFrame: number;
	let showMissFlash = false;

	/**
	 * Animation loop - runs every frame
	 * 
	 * ALL visual state is derived from rhythmEngine.getVisualState()
	 * which derives from Tone.Transport.seconds
	 */
	function animate() {
		if (rhythmEngine) {
			const state = rhythmEngine.getVisualState();
			beatPhase = state.beatPhase;
			intensity = state.intensity;
			isInWindow = state.isInWindow;
			
			// Calculate bar positions from beat phase
			// Bars meet in center at phase 0 (on beat) and phase 1 (next beat)
			// Bars are at edges at phase 0.5 (between beats)
			//
			// Phase 0.0 → 0.5: bars move from center to edges
			// Phase 0.5 → 1.0: bars move from edges to center
			
			if (beatPhase <= 0.5) {
				// Moving outward: center (50) → edges (0/100)
				const t = beatPhase * 2; // 0→1
				leftBarPos = 50 - (50 * t);   // 50→0
				rightBarPos = 50 + (50 * t);  // 50→100
			} else {
				// Moving inward: edges (0/100) → center (50)
				const t = (beatPhase - 0.5) * 2; // 0→1
				leftBarPos = 50 * t;              // 0→50
				rightBarPos = 100 - (50 * t);    // 100→50
			}
		}
		
		animationFrame = requestAnimationFrame(animate);
	}

	// Called externally when player misses
	export function showMiss() {
		showMissFlash = true;
		setTimeout(() => showMissFlash = false, 200);
	}

	onMount(() => {
		animate();
	});

	onDestroy(() => {
		if (animationFrame) {
			cancelAnimationFrame(animationFrame);
		}
	});
</script>

<div class="beat-indicator" class:miss-flash={showMissFlash}>
	<div class="beat-track">
		<!-- Left bar -->
		<div 
			class="beat-bar"
			class:in-window={isInWindow}
			style="left: {leftBarPos}%"
		></div>

		<!-- Center target -->
		<div class="beat-center" class:active={isInWindow}>
			<div 
				class="beat-heart"
				style="transform: scale({1 + intensity * 0.5}); opacity: {0.5 + intensity * 0.5}"
			>
				<svg viewBox="0 0 24 24" fill="currentColor">
					<path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
				</svg>
			</div>
		</div>

		<!-- Right bar -->
		<div 
			class="beat-bar"
			class:in-window={isInWindow}
			style="left: {rightBarPos}%"
		></div>
	</div>
</div>

<style>
	.beat-indicator {
		width: 100%;
		padding: 0.5rem;
	}

	.beat-indicator.miss-flash {
		animation: shake 0.2s ease-out;
	}

	@keyframes shake {
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
	}

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
		box-shadow: 0 0 12px rgba(139, 92, 246, 0.7);
		transition: background 0.1s ease;
	}

	.beat-bar.in-window {
		background: linear-gradient(180deg, 
			rgba(34, 197, 94, 0.9) 0%,
			rgba(74, 222, 128, 1) 50%,
			rgba(34, 197, 94, 0.9) 100%
		);
		box-shadow: 0 0 20px rgba(34, 197, 94, 0.9);
	}

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
	}

	.beat-heart {
		width: 36px;
		height: 36px;
		color: #6b7280;
		transition: color 0.1s ease;
	}

	.beat-center.active .beat-heart {
		color: #22c55e;
		filter: drop-shadow(0 0 10px rgba(34, 197, 94, 0.8));
	}

	/* Mobile */
	@media (max-width: 640px) {
		.beat-track {
			height: 48px;
		}
		.beat-bar {
			width: 6px;
			top: 6px;
			bottom: 6px;
		}
		.beat-heart {
			width: 30px;
			height: 30px;
		}
	}
</style>
