<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { RhythmEngine } from '$lib/game/rhythmEngine.js';

	export let rhythmEngine: RhythmEngine;

	let beatProgress = 0;
	let isOnBeat = false;
	let animationFrame: number;
	let showMissFlash = false;
	let heartScale = 1;
	let leftArrowPos = 0;
	let rightArrowPos = 100;

	// Animation loop
	const animate = () => {
		if (rhythmEngine) {
			const visual = rhythmEngine.getVisualFeedback();
			beatProgress = visual.beatProgress;
			isOnBeat = visual.isOnBeat;
			
			// Heart pulse effect - scale up at beat, scale down between
			// Beat happens at progress = 0 (or very close to 1)
			if (beatProgress < 0.15 || beatProgress > 0.85) {
				heartScale = 1.3;
			} else {
				heartScale = 1 + (0.3 * (1 - Math.min(beatProgress, 1 - beatProgress) * 7));
			}
			
			// Calculate arrow positions - converge to center on beat
			// Left arrow: 0% -> 45% -> 0%
			// Right arrow: 100% -> 55% -> 100%
			if (beatProgress <= 0.5) {
				// First half: moving toward center
				leftArrowPos = beatProgress * 90; // 0 -> 45
				rightArrowPos = 100 - (beatProgress * 90); // 100 -> 55
			} else {
				// Second half: moving back from edges
				leftArrowPos = (1 - beatProgress) * 90; // 45 -> 0
				rightArrowPos = 100 - ((1 - beatProgress) * 90); // 55 -> 100
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
		<!-- Left arrow (pointing right →) -->
		<div 
			class="beat-arrow left"
			style="left: {leftArrowPos}%;"
		>
			<svg viewBox="0 0 24 24" fill="currentColor">
				<path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/>
			</svg>
		</div>

		<!-- Center heart/target zone -->
		<div class="beat-center" class:on-beat={isOnBeat}>
			<div 
				class="heart"
				style="transform: scale({heartScale});"
			>
				<svg viewBox="0 0 24 24" fill="currentColor">
					<path d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"/>
				</svg>
			</div>
		</div>

		<!-- Right arrow (pointing left ←) -->
		<div 
			class="beat-arrow right"
			style="left: {rightArrowPos}%;"
		>
			<svg viewBox="0 0 24 24" fill="currentColor">
				<path d="M15.41 16.59L10.83 12l4.58-4.59L14 6l-6 6 6 6 1.41-1.41z"/>
			</svg>
		</div>
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
		width: 200px;
		height: 50px;
		background: linear-gradient(90deg, 
			rgba(139, 92, 246, 0.1) 0%, 
			rgba(139, 92, 246, 0.3) 40%, 
			rgba(34, 197, 94, 0.4) 45%,
			rgba(34, 197, 94, 0.4) 55%,
			rgba(139, 92, 246, 0.3) 60%, 
			rgba(139, 92, 246, 0.1) 100%
		);
		border-radius: 25px;
		overflow: visible;
		border: 2px solid rgba(139, 92, 246, 0.5);
	}

	.beat-arrow {
		position: absolute;
		top: 50%;
		transform: translateY(-50%) translateX(-50%);
		width: 24px;
		height: 24px;
		color: #8b5cf6;
		transition: left 0.016s linear;
		filter: drop-shadow(0 0 4px rgba(139, 92, 246, 0.5));
	}

	.beat-arrow.left svg {
		transform: scaleX(1);
	}

	.beat-arrow.right svg {
		transform: scaleX(1);
	}

	.beat-center {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 40px;
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 10;
	}

	.heart {
		width: 32px;
		height: 32px;
		color: #ef4444;
		transition: transform 0.05s ease-out;
		filter: drop-shadow(0 0 8px rgba(239, 68, 68, 0.6));
	}

	.beat-center.on-beat .heart {
		color: #22c55e;
		filter: drop-shadow(0 0 12px rgba(34, 197, 94, 0.8));
	}

	.beat-zone-indicator {
		font-size: 0.875rem;
		font-weight: bold;
		text-transform: uppercase;
		letter-spacing: 0.1em;
	}

	.beat-zone-indicator span {
		color: #ef4444;
		transition: all 0.1s ease;
	}

	.beat-zone-indicator span.active {
		color: #22c55e;
		text-shadow: 0 0 10px rgba(34, 197, 94, 0.8);
	}

	/* Responsive */
	@media (max-width: 640px) {
		.beat-track {
			width: 160px;
			height: 40px;
		}

		.beat-arrow {
			width: 20px;
			height: 20px;
		}

		.heart {
			width: 26px;
			height: 26px;
		}

		.beat-zone-indicator {
			font-size: 0.75rem;
		}
	}
</style>
