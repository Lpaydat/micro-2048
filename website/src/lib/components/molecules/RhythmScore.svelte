<script lang="ts">
	import { onMount } from 'svelte';
	
	export let rhythmScore: number = 0;
	export let combo: number = 0;
	export let maxCombo: number = 0;
	export let perfectCount: number = 0;
	export let goodCount: number = 0;
	export let missCount: number = 0;
	export let totalMoves: number = 0;

	// Calculate accuracy percentage
	$: accuracy = totalMoves > 0 ? ((perfectCount + goodCount) / totalMoves * 100).toFixed(1) : '0.0';
	
	// Get combo color based on level
	$: comboColor = combo >= 10 ? 'text-purple-600' : combo >= 5 ? 'text-blue-600' : combo >= 3 ? 'text-green-600' : 'text-gray-600';
	
	// Get rhythm grade
	$: rhythmGrade = getRhythmGrade(parseFloat(accuracy));

	function getRhythmGrade(accuracyPercent: number): { grade: string; color: string } {
		if (accuracyPercent >= 95) return { grade: 'S', color: 'text-purple-600' };
		if (accuracyPercent >= 90) return { grade: 'A', color: 'text-blue-600' };
		if (accuracyPercent >= 80) return { grade: 'B', color: 'text-green-600' };
		if (accuracyPercent >= 70) return { grade: 'C', color: 'text-yellow-600' };
		if (accuracyPercent >= 60) return { grade: 'D', color: 'text-orange-600' };
		return { grade: 'F', color: 'text-red-600' };
	}

	// Animation for score changes
	let scoreChange: { value: number; type: 'perfect' | 'good' | 'miss' } | null = null;
	let showScoreChange = false;

	export function showScoreChangeAnimation(value: number, type: 'perfect' | 'good' | 'miss') {
		scoreChange = { value, type };
		showScoreChange = true;
		
		setTimeout(() => {
			showScoreChange = false;
		}, 1000);
	}

	// Combo animation
	let comboAnimation = false;
	export function triggerComboAnimation() {
		comboAnimation = true;
		setTimeout(() => {
			comboAnimation = false;
		}, 300);
	}
</script>

<div class="rhythm-score">
	<!-- Main Score Display -->
	<div class="score-display">
		<div class="rhythm-grade {rhythmGrade.color}">
			{rhythmGrade.grade}
		</div>
		<div class="score-details">
			<div class="accuracy">{accuracy}%</div>
			<div class="moves">{totalMoves} moves</div>
		</div>
	</div>

	<!-- Combo Display -->
	<div class="combo-display {comboAnimation ? 'animate-pulse' : ''}">
		<div class="combo-label">COMBO</div>
		<div class="combo-value {comboColor}">{combo}x</div>
		{#if combo > 0}
			<div class="combo-bar">
				<div 
					class="combo-fill"
					style="width: {Math.min(combo * 10, 100)}%; background: {combo >= 10 ? '#9333ea' : combo >= 5 ? '#3b82f6' : combo >= 3 ? '#10b981' : '#6b7280'};"
				></div>
			</div>
		{/if}
	</div>

	<!-- Stats Breakdown -->
	<div class="stats-breakdown">
		<div class="stat-item">
			<span class="stat-value text-green-600">{perfectCount}</span>
			<span class="stat-label">Perfect</span>
		</div>
		<div class="stat-item">
			<span class="stat-value text-blue-600">{goodCount}</span>
			<span class="stat-label">Good</span>
		</div>
		<div class="stat-item">
			<span class="stat-value text-red-600">{missCount}</span>
			<span class="stat-label">Miss</span>
		</div>
		<div class="stat-item">
			<span class="stat-value text-purple-600">{maxCombo}</span>
			<span class="stat-label">Best</span>
		</div>
	</div>

	<!-- Score Change Animation -->
	{#if showScoreChange && scoreChange}
		<div class="score-change {scoreChange.type} animate-bounce">
			{#if scoreChange.type === 'perfect'}
				<span class="text-green-600">+{scoreChange.value} PERFECT!</span>
			{:else if scoreChange.type === 'good'}
				<span class="text-blue-600">+{scoreChange.value} GOOD!</span>
			{:else}
				<span class="text-red-600">MISS!</span>
			{/if}
		</div>
	{/if}
</div>

<style>
	.rhythm-score {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		padding: 0.75rem;
		background: rgba(249, 250, 251, 0.9);
		border-radius: 0.5rem;
		border: 1px solid rgba(156, 163, 175, 0.3);
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
	}

	/* Score Display */
	.score-display {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 1rem;
	}

	.rhythm-grade {
		font-size: 2rem;
		font-weight: bold;
		line-height: 1;
	}

	.score-details {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 0.125rem;
	}

	.accuracy {
		font-size: 1.25rem;
		font-weight: 600;
		color: #1f2937;
	}

	.moves {
		font-size: 0.75rem;
		color: #6b7280;
	}

	/* Combo Display */
	.combo-display {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
	}

	.combo-label {
		font-size: 0.625rem;
		font-weight: 600;
		color: #6b7280;
		letter-spacing: 0.05em;
	}

	.combo-value {
		font-size: 1.5rem;
		font-weight: bold;
		line-height: 1;
		transition: color 0.2s ease;
	}

	.combo-bar {
		width: 100%;
		height: 4px;
		background: #e5e7eb;
		border-radius: 2px;
		overflow: hidden;
	}

	.combo-fill {
		height: 100%;
		transition: all 0.3s ease;
		border-radius: 2px;
	}

	/* Stats Breakdown */
	.stats-breakdown {
		display: grid;
		grid-template-columns: repeat(4, 1fr);
		gap: 0.5rem;
	}

	.stat-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.125rem;
	}

	.stat-value {
		font-size: 1rem;
		font-weight: 600;
		line-height: 1;
	}

	.stat-label {
		font-size: 0.625rem;
		color: #6b7280;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	/* Score Change Animation */
	.score-change {
		position: absolute;
		top: -2rem;
		left: 50%;
		transform: translateX(-50%);
		padding: 0.25rem 0.75rem;
		border-radius: 1rem;
		font-weight: bold;
		font-size: 0.875rem;
		white-space: nowrap;
		z-index: 10;
		background: rgba(255, 255, 255, 0.95);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
	}

	/* Responsive */
	@media (max-width: 640px) {
		.rhythm-score {
			padding: 0.5rem;
			gap: 0.5rem;
		}

		.rhythm-grade {
			font-size: 1.5rem;
		}

		.accuracy {
			font-size: 1rem;
		}

		.combo-value {
			font-size: 1.25rem;
		}

		.stats-breakdown {
			gap: 0.25rem;
		}

		.stat-value {
			font-size: 0.875rem;
		}

		.stat-label {
			font-size: 0.5rem;
		}
	}
</style>