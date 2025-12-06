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
	
	// Single bar on each side - one bar per beat
	let leftBarPos = 0;
	let rightBarPos = 100;

	// Animation loop
	const animate = () => {
		if (rhythmEngine) {
			const visual = rhythmEngine.getVisualFeedback();
			beatProgress = visual.beatProgress;
			isOnBeat = visual.isOnBeat;
			
			// Heart pulse effect - quick pulse at beat moment (when progress is near 0)
			if (beatProgress < 0.1 || beatProgress > 0.95) {
				heartScale = 1.5;
				heartBeat = true;
			} else if (beatProgress < 0.2) {
				heartScale = 1.3;
				heartBeat = true;
			} else {
				heartScale = 1;
				heartBeat = false;
			}
			
			// Bar positions: move from edge to center over ONE beat
			// At progress=0, bars are at center (beat moment)
			// At progress=0.5, bars are at edges
			// At progress=1, bars are back at center (next beat)
			
			// We want bars to continuously move INWARD and reset
			// Left bar: starts at 0%, moves to 50% (center)
			// Right bar: starts at 100%, moves to 50% (center)
			
			// Use beatProgress directly - bar travels full distance in one beat
			leftBarPos = beatProgress * 50;  // 0% -> 50%
			rightBarPos = 100 - (beatProgress * 50);  // 100% -> 50%
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
	
	// Determine if bar is in the "hit zone" (close to center)
	$: leftApproaching = leftBarPos > 35;
	$: rightApproaching = rightBarPos < 65;
</script>

<div class="beat-indicator" class:miss-flash={showMissFlash}>
	<!-- Track background -->
	<div class="beat-track">
		<!-- Left bar moving right -->
		<div 
			class="beat-bar left"
			class:approaching={leftApproaching}
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
			class:approaching={rightApproaching}
			style="left: {rightBarPos}%;"
		></div>
	</div>

	<!-- Beat zone indicator -->
	<div class="beat-zone-indicator">
		<span class:active={isOnBeat}>
			{isOnBeat ? 'GO!' : 'WAIT'}
		</span>
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
		max-width: 400px;
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

	.beat-zone-indicator {
		font-size: 1rem;
		font-weight: bold;
		text-transform: uppercase;
		letter-spacing: 0.15em;
	}

	.beat-zone-indicator span {
		color: #6b7280;
		transition: all 0.1s ease;
	}

	.beat-zone-indicator span.active {
		color: #22c55e;
		text-shadow: 0 0 12px rgba(34, 197, 94, 0.9);
	}

	/* Responsive */
	@media (max-width: 640px) {
		.beat-track {
			max-width: 320px;
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

		.beat-zone-indicator {
			font-size: 0.875rem;
		}
	}
</style>
