<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { RhythmEngine } from '$lib/game/rhythmEngine.js';

	export let rhythmEngine: RhythmEngine;

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
</div>

<style>
	.beat-indicator {
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
	}
</style>
