<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import type { RhythmEngine, RhythmFeedback } from '$lib/game/rhythmEngine.js';

	export let rhythmEngine: RhythmEngine | null = null;
	export let showFeedback: boolean = true;
	export let compact: boolean = false;

	let feedback: RhythmFeedback | null = null;
	let visualData: {
		isOnBeat: boolean;
		beatProgress: number;
		timeToNext: number;
		intensity: number;
	} = {
		isOnBeat: false,
		beatProgress: 0,
		timeToNext: 0,
		intensity: 0
	};

	let animationFrame: number;
	let lastFeedbackTime: number = 0;

	// Animation loop for visual updates
	const animate = () => {
		if (rhythmEngine) {
			visualData = rhythmEngine.getVisualFeedback();
		}
		animationFrame = requestAnimationFrame(animate);
	};

	// Show rhythm feedback when move is made
	export function showMoveFeedback(moveFeedback: RhythmFeedback) {
		feedback = moveFeedback;
		lastFeedbackTime = Date.now();
		
		// Clear feedback after 1 second
		setTimeout(() => {
			if (Date.now() - lastFeedbackTime >= 1000) {
				feedback = null;
			}
		}, 1000);
	}

	onMount(() => {
		animate();
	});

	onDestroy(() => {
		if (animationFrame) {
			cancelAnimationFrame(animationFrame);
		}
	});

	// Get feedback color and text
	function getFeedbackStyle(accuracy: RhythmFeedback['accuracy']) {
		switch (accuracy) {
			case 'perfect':
				return { color: 'text-green-600', bg: 'bg-green-100', text: 'PERFECT!', icon: '🎯' };
			case 'good':
				return { color: 'text-blue-600', bg: 'bg-blue-100', text: 'GOOD!', icon: '👍' };
			case 'early':
				return { color: 'text-yellow-600', bg: 'bg-yellow-100', text: 'EARLY!', icon: '⏰' };
			case 'late':
				return { color: 'text-orange-600', bg: 'bg-orange-100', text: 'LATE!', icon: '⏰' };
			case 'miss':
				return { color: 'text-red-600', bg: 'bg-red-100', text: 'MISS!', icon: '❌' };
			default:
				return { color: 'text-gray-600', bg: 'bg-gray-100', text: '', icon: '' };
		}
	}

	// Calculate pulse animation
	function getPulseStyle(): string {
		if (!rhythmEngine || !visualData.isOnBeat) return '';
		
		const scale = 1 + (visualData.intensity * 0.1);
		const opacity = 0.3 + (visualData.intensity * 0.7);
		
		return `transform: scale(${scale}); opacity: ${opacity};`;
	}

	// Get beat indicator color
	function getBeatColor() {
		if (!rhythmEngine) return 'bg-gray-300';
		
		if (visualData.intensity > 0.8) return 'bg-purple-500';
		if (visualData.intensity > 0.5) return 'bg-purple-400';
		if (visualData.intensity > 0.2) return 'bg-purple-300';
		return 'bg-purple-200';
	}
</script>

{#if rhythmEngine}
	<div class="rhythm-indicator {compact ? 'compact' : 'full'}">
		<!-- Beat Progress Bar -->
		<div class="beat-progress-container">
			<div class="beat-progress-bar">
				<div 
					class="beat-progress-fill"
					style="width: {visualData.beatProgress * 100}%; background: {getBeatColor()};"
				></div>
				<!-- Beat marker -->
				<div 
					class="beat-marker"
					style="left: {visualData.beatProgress * 100}%;"
				></div>
			</div>
		</div>

		<!-- Visual Pulse Indicator -->
		<div class="pulse-container">
			<div 
				class="pulse-ring"
				style={getPulseStyle()}
			></div>
			<div class="pulse-center"></div>
		</div>

		<!-- Feedback Display -->
		{#if showFeedback && feedback}
			<div class="feedback-display {getFeedbackStyle(feedback.accuracy).bg}">
				<span class="feedback-icon">{getFeedbackStyle(feedback.accuracy).icon}</span>
				<span class="feedback-text {getFeedbackStyle(feedback.accuracy).color}">
					{getFeedbackStyle(feedback.accuracy).text}
				</span>
				<span class="feedback-timing text-xs text-gray-600">
					{Math.abs(feedback.timingDiff).toFixed(0)}ms
				</span>
			</div>
		{/if}

		<!-- Compact Mode Info -->
		{#if compact}
			<div class="compact-info">
				<div class="beat-indicator {getBeatColor()}"></div>
				<span class="text-xs text-gray-600">
					{Math.round(visualData.timeToNext)}ms
				</span>
			</div>
		{/if}
	</div>
{/if}

<style>
	.rhythm-indicator {
		position: relative;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.5rem;
	}

	.rhythm-indicator.compact {
		flex-direction: row;
		gap: 0.25rem;
	}

	/* Beat Progress Bar */
	.beat-progress-container {
		width: 100%;
		max-width: 200px;
	}

	.beat-progress-bar {
		position: relative;
		width: 100%;
		height: 8px;
		background: #e5e7eb;
		border-radius: 4px;
		overflow: hidden;
	}

	.beat-progress-fill {
		height: 100%;
		transition: width 0.05s linear;
		border-radius: 4px;
	}

	.beat-marker {
		position: absolute;
		top: -2px;
		width: 4px;
		height: 12px;
		background: #4b5563;
		border-radius: 2px;
		transform: translateX(-50%);
	}

	/* Pulse Indicator */
	.pulse-container {
		position: relative;
		width: 60px;
		height: 60px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.pulse-ring {
		position: absolute;
		width: 100%;
		height: 100%;
		border: 3px solid #9333ea;
		border-radius: 50%;
		transition: all 0.1s ease-out;
	}

	.pulse-center {
		width: 20px;
		height: 20px;
		background: #9333ea;
		border-radius: 50%;
		z-index: 1;
	}

	/* Feedback Display */
	.feedback-display {
		position: absolute;
		top: -40px;
		left: 50%;
		transform: translateX(-50%);
		padding: 0.25rem 0.5rem;
		border-radius: 0.25rem;
		display: flex;
		align-items: center;
		gap: 0.25rem;
		font-weight: bold;
		font-size: 0.75rem;
		white-space: nowrap;
		z-index: 10;
		animation: feedbackPop 0.3s ease-out;
	}

	@keyframes feedbackPop {
		0% {
			transform: translateX(-50%) scale(0.8);
			opacity: 0;
		}
		50% {
			transform: translateX(-50%) scale(1.1);
		}
		100% {
			transform: translateX(-50%) scale(1);
			opacity: 1;
		}
	}

	.feedback-icon {
		font-size: 1rem;
	}

	.feedback-timing {
		opacity: 0.7;
	}

	/* Compact Mode */
	.compact-info {
		display: flex;
		align-items: center;
		gap: 0.25rem;
	}

	.beat-indicator {
		width: 12px;
		height: 12px;
		border-radius: 50%;
		transition: all 0.1s ease-out;
	}

	/* Responsive */
	@media (max-width: 640px) {
		.beat-progress-container {
			max-width: 150px;
		}
		
		.pulse-container {
			width: 50px;
			height: 50px;
		}
		
		.pulse-center {
			width: 16px;
			height: 16px;
		}
	}
</style>