<script lang="ts">
	import { createEventDispatcher } from 'svelte';
	import type { RhythmSettings } from '$lib/game/rhythmEngine.js';

	export let settings: RhythmSettings;
	export let showAdvanced: boolean = false;

	const dispatch = createEventDispatcher();

	function updateSetting(key: keyof RhythmSettings, value: any) {
		settings = { ...settings, [key]: value };
		dispatch('settingsChange', settings);
	}

	function toggleAdvanced() {
		showAdvanced = !showAdvanced;
	}

	// Preset configurations
	const presets = [
		{ name: 'Casual', bpm: 100, tolerance: 200 },
		{ name: 'Normal', bpm: 120, tolerance: 150 },
		{ name: 'Challenge', bpm: 140, tolerance: 100 },
		{ name: 'Expert', bpm: 160, tolerance: 75 },
		{ name: 'Master', bpm: 180, tolerance: 50 }
	];

	function applyPreset(preset: typeof presets[0]) {
		updateSetting('bpm', preset.bpm);
		updateSetting('tolerance', preset.tolerance);
	}
</script>

<div class="rhythm-settings">
	<div class="settings-header">
		<h3 class="settings-title">🎵 Rhythm Settings</h3>
		<button 
			class="advanced-toggle"
			onclick={toggleAdvanced}
			type="button"
		>
			{showAdvanced ? '▼ Simple' : '▲ Advanced'}
		</button>
	</div>

	<!-- Basic Settings -->
	<div class="settings-section">
		<div class="setting-group">
			<label class="setting-label">
				<input
					type="checkbox"
					checked={settings.enabled}
					onchange={(e) => updateSetting('enabled', (e.target as HTMLInputElement).checked)}
					class="setting-checkbox"
				/>
				Enable Rhythm Mode
			</label>
			<p class="setting-description">
				Move on the beat like Crypt of the Necrodancer!
			</p>
		</div>

		{#if settings.enabled}
			<!-- Presets -->
			<div class="setting-group">
				<label class="setting-label">Difficulty Presets</label>
				<div class="presets-grid">
					{#each presets as preset}
						<button
							class="preset-button"
							class:selected={settings.bpm === preset.bpm && settings.tolerance === preset.tolerance}
							onclick={() => applyPreset(preset)}
							type="button"
						>
							{preset.name}
							<div class="preset-details">
								{preset.bpm} BPM • {preset.tolerance}ms
							</div>
						</button>
					{/each}
				</div>
			</div>

			<!-- BPM Slider -->
			<div class="setting-group">
				<label class="setting-label">
					Beats Per Minute: <span class="setting-value">{settings.bpm}</span>
				</label>
				<input
					type="range"
					min="60"
					max="200"
					bind:value={settings.bpm}
					oninput={(e) => updateSetting('bpm', parseInt((e.target as HTMLInputElement).value))}
					class="setting-slider"
				/>
				<div class="slider-labels">
					<span>Slow (60)</span>
					<span>Normal (120)</span>
					<span>Fast (200)</span>
				</div>
			</div>

			<!-- Tolerance Slider -->
			<div class="setting-group">
				<label class="setting-label">
					Timing Tolerance: <span class="setting-value">{settings.tolerance}ms</span>
				</label>
				<input
					type="range"
					min="50"
					max="500"
					step="10"
					bind:value={settings.tolerance}
					oninput={(e) => updateSetting('tolerance', parseInt((e.target as HTMLInputElement).value))}
					class="setting-slider"
				/>
				<div class="slider-labels">
					<span>Strict (50ms)</span>
					<span>Normal (150ms)</span>
					<span>Relaxed (500ms)</span>
				</div>
			</div>
		{/if}
	</div>

	<!-- Advanced Settings -->
	{#if showAdvanced && settings.enabled}
		<div class="settings-section advanced">
			<h4 class="section-title">Advanced Options</h4>
			
			<div class="setting-group">
				<label class="setting-label">Visual Feedback</label>
				<div class="checkbox-group">
					<label class="checkbox-item">
						<input type="checkbox" checked={true} disabled />
						Show beat indicator
					</label>
					<label class="checkbox-item">
						<input type="checkbox" checked={true} disabled />
						Show timing feedback
					</label>
					<label class="checkbox-item">
						<input type="checkbox" checked={true} disabled />
						Animate on beat
					</label>
				</div>
			</div>

			<div class="setting-group">
				<label class="setting-label">Audio Settings</label>
				<div class="checkbox-group">
					<label class="checkbox-item">
						<input type="checkbox" checked={true} disabled />
						Metronome sounds
					</label>
					<label class="checkbox-item">
						<input type="checkbox" checked={true} disabled />
						Downbeat accent
					</label>
				</div>
			</div>

			<div class="setting-group">
				<label class="setting-label">Scoring</label>
				<div class="info-text">
					<p>• Perfect: ±50ms (100 points)</p>
					<p>• Good: ±100ms (50 points)</p>
					<p>• Early/Late: Within tolerance (25 points)</p>
					<p>• Miss: Outside tolerance (0 points)</p>
					<p>• Combo bonus: +10% per consecutive hit</p>
				</div>
			</div>
		</div>
	{/if}

	<!-- Tips -->
	<div class="tips-section">
		<h4 class="section-title">🎯 Tips</h4>
		<ul class="tips-list">
			<li>Watch the visual beat indicator to time your moves</li>
			<li>Listen for the metronome clicks (if audio enabled)</li>
			<li>Start with slower BPM and higher tolerance to learn</li>
			<li>Build combos for score multipliers</li>
			<li>Perfect timing gives maximum points and combo bonus</li>
		</ul>
	</div>
</div>

<style>
	.rhythm-settings {
		display: flex;
		flex-direction: column;
		gap: 1rem;
		padding: 1rem;
		background: #f9fafb;
		border-radius: 0.5rem;
		border: 1px solid #e5e7eb;
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
	}

	.settings-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.settings-title {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 600;
		color: #1f2937;
	}

	.advanced-toggle {
		background: none;
		border: 1px solid #d1d5db;
		border-radius: 0.25rem;
		padding: 0.25rem 0.5rem;
		font-size: 0.75rem;
		color: #6b7280;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.advanced-toggle:hover {
		background: #f3f4f6;
		color: #374151;
	}

	.settings-section {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}

	.settings-section.advanced {
		padding-top: 1rem;
		border-top: 1px solid #e5e7eb;
	}

	.section-title {
		margin: 0;
		font-size: 0.875rem;
		font-weight: 600;
		color: #374151;
	}

	.setting-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.setting-label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		font-weight: 500;
		color: #374151;
	}

	.setting-value {
		font-weight: 600;
		color: #7c3aed;
	}

	.setting-checkbox {
		width: 1rem;
		height: 1rem;
		color: #7c3aed;
	}

	.setting-description {
		margin: 0;
		font-size: 0.75rem;
		color: #6b7280;
		line-height: 1.4;
	}

	.setting-slider {
		width: 100%;
		height: 6px;
		border-radius: 3px;
		background: #e5e7eb;
		outline: none;
		-webkit-appearance: none;
		appearance: none;
	}

	.setting-slider::-webkit-slider-thumb {
		-webkit-appearance: none;
		appearance: none;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: #7c3aed;
		cursor: pointer;
		border: 2px solid white;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.setting-slider::-moz-range-thumb {
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: #7c3aed;
		cursor: pointer;
		border: 2px solid white;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.slider-labels {
		display: flex;
		justify-content: space-between;
		font-size: 0.75rem;
		color: #6b7280;
	}

	/* Presets */
	.presets-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
		gap: 0.5rem;
	}

	.preset-button {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 0.25rem;
		padding: 0.5rem;
		border: 1px solid #d1d5db;
		border-radius: 0.375rem;
		background: white;
		cursor: pointer;
		transition: all 0.2s ease;
		font-size: 0.75rem;
	}

	.preset-button:hover {
		background: #f9fafb;
		border-color: #7c3aed;
	}

	.preset-button.selected {
		background: #ede9fe;
		border-color: #7c3aed;
		color: #7c3aed;
		font-weight: 600;
	}

	.preset-details {
		font-size: 0.625rem;
		color: #6b7280;
	}

	/* Checkboxes */
	.checkbox-group {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.checkbox-item {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.875rem;
		color: #374151;
	}

	.checkbox-item input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.info-text {
		font-size: 0.75rem;
		color: #6b7280;
		line-height: 1.4;
	}

	.info-text p {
		margin: 0.25rem 0;
	}

	/* Tips */
	.tips-section {
		padding-top: 1rem;
		border-top: 1px solid #e5e7eb;
	}

	.tips-list {
		margin: 0;
		padding-left: 1.25rem;
		font-size: 0.75rem;
		color: #6b7280;
		line-height: 1.5;
	}

	.tips-list li {
		margin-bottom: 0.25rem;
	}

	/* Responsive */
	@media (max-width: 640px) {
		.rhythm-settings {
			padding: 0.75rem;
			gap: 0.75rem;
		}

		.presets-grid {
			grid-template-columns: repeat(2, 1fr);
		}

		.slider-labels {
			font-size: 0.625rem;
		}
	}
</style>