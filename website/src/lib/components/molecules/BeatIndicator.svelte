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
	
	// Track multiple bars (we show bars for current and next few beats)
	// Each bar position is 0-100 where 50 is center
	let leftBars: number[] = [];
	let rightBars: number[] = [];

	// Animation loop
	const animate = () => {
		if (rhythmEngine) {
			const visual = rhythmEngine.getVisualFeedback();
			beatProgress = visual.beatProgress;
			isOnBeat = visual.isOnBeat;
			
			// Heart pulse effect - quick pulse at beat moment
			if (beatProgress < 0.1) {
				heartScale = 1.4;
				heartBeat = true;
			} else if (beatProgress < 0.2) {
				heartScale = 1.2;
				heartBeat = true;
			} else {
				heartScale = 1;
				heartBeat = false;
			}
			
			// Calculate bar positions
			// Bars move from edge (0%) to center (50%) over one beat cycle
			// We show 3 bars on each side, offset by 1/3 of a beat
			const barsPerSide = 3;
			const newLeftBars: number[] = [];
			const newRightBars: number[] = [];
			
			for (let i = 0; i < barsPerSide; i++) {
				// Each bar is offset by 1/barsPerSide of the beat cycle
				const offset = i / barsPerSide;
				let barProgress = (beatProgress + offset) % 1;
				
				// Left bar: moves from 0% to 50%
				const leftPos = barProgress * 50;
				newLeftBars.push(leftPos);
				
				// Right bar: moves from 100% to 50%
				const rightPos = 100 - (barProgress * 50);
				newRightBars.push(rightPos);
			}
			
			leftBars = newLeftBars;
			rightBars = newRightBars;
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
		<!-- Left bars moving right -->
		{#each leftBars as pos, i}
			<div 
				class="beat-bar left"
				class:approaching={pos > 40}
				style="left: {pos}%;"
			></div>
		{/each}

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

		<!-- Right bars moving left -->
		{#each rightBars as pos, i}
			<div 
				class="beat-bar right"
				class:approaching={pos < 60}
				style="left: {pos}%;"
			></div>
		{/each}
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
		width: 280px;
		height: 50px;
		background: linear-gradient(90deg, 
			rgba(30, 30, 40, 0.9) 0%,
			rgba(40, 40, 55, 0.9) 35%,
			rgba(34, 197, 94, 0.3) 45%,
			rgba(34, 197, 94, 0.3) 55%,
			rgba(40, 40, 55, 0.9) 65%,
			rgba(30, 30, 40, 0.9) 100%
		);
		border-radius: 8px;
		overflow: hidden;
		border: 2px solid rgba(139, 92, 246, 0.5);
		box-shadow: inset 0 0 20px rgba(0, 0, 0, 0.5);
	}

	/* Beat bars */
	.beat-bar {
		position: absolute;
		top: 10%;
		height: 80%;
		width: 6px;
		background: linear-gradient(180deg, 
			rgba(139, 92, 246, 0.9) 0%,
			rgba(168, 85, 247, 1) 50%,
			rgba(139, 92, 246, 0.9) 100%
		);
		border-radius: 3px;
		transform: translateX(-50%);
		box-shadow: 0 0 10px rgba(139, 92, 246, 0.6);
		transition: none;
	}

	.beat-bar.approaching {
		background: linear-gradient(180deg, 
			rgba(34, 197, 94, 0.9) 0%,
			rgba(74, 222, 128, 1) 50%,
			rgba(34, 197, 94, 0.9) 100%
		);
		box-shadow: 0 0 15px rgba(34, 197, 94, 0.8);
	}

	/* Center target zone */
	.beat-center {
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 50px;
		height: 50px;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 10;
		background: radial-gradient(circle, rgba(0,0,0,0.3) 0%, transparent 70%);
		border-radius: 50%;
	}

	.heart {
		width: 32px;
		height: 32px;
		color: #6b7280;
		transition: transform 0.08s ease-out, color 0.1s ease;
		filter: drop-shadow(0 0 4px rgba(107, 114, 128, 0.4));
	}

	.heart.beating {
		color: #ef4444;
		filter: drop-shadow(0 0 12px rgba(239, 68, 68, 0.8));
	}

	.beat-center.on-beat .heart {
		color: #22c55e;
		filter: drop-shadow(0 0 15px rgba(34, 197, 94, 0.9));
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
			width: 220px;
			height: 40px;
		}

		.beat-bar {
			width: 5px;
		}

		.beat-center {
			width: 40px;
			height: 40px;
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
