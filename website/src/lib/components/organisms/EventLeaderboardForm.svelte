<script lang="ts">
	import { fromZonedTime } from 'date-fns-tz';
	import Input from '../atoms/Input.svelte';
	import Button from '../atoms/Button.svelte';
	import { preventDefault } from '$lib/utils/preventDefault';
	import {
		createLeaderboard,
		type LeaderboardSettings
	} from '$lib/graphql/mutations/leaderboardAction.ts';
	import { getModalStore } from '@skeletonlabs/skeleton';
	import { getClient } from '$lib/client';
	import { chainId as mainChainId } from '$lib/constants';
	import { userStore } from '$lib/stores/userStore';
	import { MUSIC_TRACKS } from '$lib/game/rhythmEngine.js';

	const client = getClient(mainChainId, true);
	const modalStore = getModalStore();

	let name = $state('');
	let startTime = $state('');
	let endTime = $state('');
	let description = $state('');
	let shardNumber = $state(4);
	let baseTriggererCount = $state(5); // Changed from 2 to 5
	let loading = $state(false);
	let errorMessage = $state('');
	let noStartLimit = $state(false);
	let noEndLimit = $state(false);
	
	// Rhythm mode settings
	let rhythmMode = $state(false);
	let rhythmBPM = $state(120);
	let rhythmTolerance = $state(150); // milliseconds
	let rhythmUseMusic = $state(true); // Use music instead of metronome
	let rhythmTrack = $state('random'); // 'random' or track index ('0', '1', '2')
	
	// Auto-update BPM when track changes (only for specific tracks)
	$effect(() => {
		if (rhythmUseMusic && rhythmTrack !== 'random') {
			const trackIndex = parseInt(rhythmTrack);
			if (!isNaN(trackIndex) && MUSIC_TRACKS[trackIndex]) {
				rhythmBPM = MUSIC_TRACKS[trackIndex].bpm;
			}
		}
	});

	// Check if current user is admin
	const isAdmin = $derived($userStore.isMod === true);

	// Set default times (1 hour from now to 24 hours from now)
	const setDefaultTimes = () => {
		const now = new Date();
		const oneHourLater = new Date(now.getTime() + 60 * 60 * 1000);
		const oneDayLater = new Date(now.getTime() + 24 * 60 * 60 * 1000);

		startTime = oneHourLater.toISOString().slice(0, 16);
		endTime = oneDayLater.toISOString().slice(0, 16);
	};

	// Set defaults on component mount
	if (typeof window !== 'undefined') {
		setDefaultTimes();
	}

	const handleSubmit = async () => {
		loading = true;
		errorMessage = '';
		const eventName = name.trim().replace(/\s+/g, ' ');

		try {
			// Check if user is logged in
			const username = localStorage.getItem('username');
			const passwordHash = localStorage.getItem('passwordHash');

			if (!username || !passwordHash) {
				alert('You must be logged in to create a tournament. Please register/login first.');
				return;
			}

			// Validate inputs
			if (!eventName) {
				errorMessage = 'Name cannot be empty.';
				return;
			}

			// Set default times if not provided and not unlimited
			if (!noStartLimit && !startTime) {
				const now = new Date();
				const oneHourLater = new Date(now.getTime() + 60 * 60 * 1000);
				startTime = oneHourLater.toISOString().slice(0, 16);
			}
			if (!noEndLimit && !endTime) {
				const now = new Date();
				const oneDayLater = new Date(now.getTime() + 24 * 60 * 60 * 1000);
				endTime = oneDayLater.toISOString().slice(0, 16);
			}

			// Validate times only if not unlimited
			if (!noStartLimit && !noEndLimit && new Date(startTime) >= new Date(endTime)) {
				errorMessage = 'Start time must be before end time.';
				return;
			}

			// Only validate admin fields if user is admin
			if (isAdmin) {
				if (shardNumber < 1 || shardNumber > 20) {
					errorMessage = 'Shard number must be between 1 and 20.';
					return;
				}

				if (baseTriggererCount < 1 || baseTriggererCount > 10) {
					errorMessage = 'Base triggerer count must be between 1 and 10.';
					return;
				}
			}

			const userTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
			const settings: LeaderboardSettings = {
				name: eventName,
				description: description || undefined,
				startTime: noStartLimit
					? '0'
					: fromZonedTime(new Date(startTime), userTimeZone).getTime().toString(),
				endTime: noEndLimit
					? '0'
					: fromZonedTime(new Date(endTime), userTimeZone).getTime().toString(),
				// Only include admin fields if user is admin
				...(isAdmin && {
					shardNumber: shardNumber,
					baseTriggererCount: baseTriggererCount
				}),
				// Rhythm mode settings (stored in description for now since backend doesn't support them yet)
				// For random track with music, BPM is determined by the track, not the slider
				...(rhythmMode && {
					description: `${description || ''} [RHYTHM_MODE:true,BPM:${rhythmUseMusic && rhythmTrack === 'random' ? 0 : rhythmBPM},TOLERANCE:${rhythmTolerance},MUSIC:${rhythmUseMusic},TRACK:${rhythmTrack}]`.trim()
				})
			};

			const result = createLeaderboard(client, settings);

			if (!result) {
				errorMessage = 'Authentication failed. Please make sure you are logged in.';
				return;
			}

			// Subscribe to the result to catch errors
			result.subscribe(($result: any) => {
				if ($result.error) {
					console.error('Tournament creation error:', $result.error);
					console.error('Error details:', JSON.stringify($result.error, null, 2));
					errorMessage =
						'Failed to create tournament. Please check your credentials and try again.';
				} else if ($result.data) {
					modalStore.close();
				}
			});
		} catch (error) {
			console.error('Unexpected error:', error);
			errorMessage = 'An unexpected error occurred. Please try again.';
		} finally {
			loading = false;
		}
	};
</script>

<form
	onsubmit={preventDefault(handleSubmit)}
	class="mx-auto w-full max-w-md rounded-md bg-[#FAF8EF] p-6 shadow-md max-h-[85vh] overflow-y-auto"
>
	<div class="space-y-6">
		<!-- Title -->
		<div class="text-center">
			<h2 class="game-font text-2xl font-bold text-[#776E65]">Event Details</h2>
		</div>

		<!-- Name Field -->
		<div class="form-field">
			<Input
				id="name"
				label="Name"
				bind:value={name}
				placeholder="Enter name (max 25 chars)"
				required
				maxlength={25}
				disabled={loading}
			/>
		</div>

		<!-- Start Time Field -->
		<div class="form-field">
			<div class="mb-2 flex items-center gap-2">
				<input
					type="checkbox"
					id="noStartLimit"
					bind:checked={noStartLimit}
					disabled={loading}
					class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
				/>
				<label for="noStartLimit" class="text-sm font-medium text-gray-700">
					No start time limit (start immediately)
				</label>
			</div>
			<Input
				id="startTime"
				label="Start Time"
				bind:value={startTime}
				placeholder="Select start time"
				type="datetime-local"
				disabled={loading || noStartLimit}
			/>
			{#if !noStartLimit}
				<p class="mt-1 text-xs text-gray-600">Defaults to 1 hour from now if empty</p>
			{/if}
		</div>

		<!-- End Time Field -->
		<div class="form-field">
			<div class="mb-2 flex items-center gap-2">
				<input
					type="checkbox"
					id="noEndLimit"
					bind:checked={noEndLimit}
					disabled={loading}
					class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
				/>
				<label for="noEndLimit" class="text-sm font-medium text-gray-700">
					No end time limit (runs indefinitely)
				</label>
			</div>
			<Input
				id="endTime"
				label="End Time"
				bind:value={endTime}
				placeholder="Select end time"
				type="datetime-local"
				disabled={loading || noEndLimit}
			/>
			{#if !noEndLimit}
				<p class="mt-1 text-xs text-gray-600">Defaults to 24 hours from now if empty</p>
			{/if}
			{#if !noStartLimit && !noEndLimit}
				<button
					type="button"
					onclick={setDefaultTimes}
					class="mt-2 text-xs text-blue-600 underline hover:text-blue-800"
					disabled={loading}
				>
					Reset to default times
				</button>
			{/if}
		</div>

		<!-- Shard Number Field - Admin Only -->
		{#if isAdmin}
			<div class="form-field">
				<Input
					id="shardNumber"
					label="Number of Shards (1-20)"
					bind:value={shardNumber}
					placeholder="Number of game shards"
					type="number"
					min="1"
					max="20"
					disabled={loading}
				/>
				<p class="mt-1 text-xs text-gray-600">
					Number of parallel game chains. More shards = higher capacity but increased complexity.
				</p>
			</div>
		{/if}

		<!-- Base Triggerer Count Field - Admin Only -->
		{#if isAdmin}
			<div class="form-field">
				<Input
					id="baseTriggererCount"
					label="Base Triggerer Count (1-10)"
					bind:value={baseTriggererCount}
					placeholder="Number of triggerers for aggregation"
					type="number"
					min="1"
					max="10"
					disabled={loading}
				/>
				<p class="mt-1 text-xs text-gray-600">
					Number of chains authorized to trigger score aggregation. Higher count = more redundancy.
				</p>
			</div>
		{/if}

		<!-- Rhythm Mode Section -->
		<div class="form-field">
			<div class="mb-4 rounded-md bg-purple-50 p-4 border border-purple-200">
				<div class="mb-3 flex items-center gap-2">
					<input
						type="checkbox"
						id="rhythmMode"
						bind:checked={rhythmMode}
						disabled={loading}
						class="h-4 w-4 rounded border-purple-300 text-purple-600 focus:ring-purple-500"
					/>
					<label for="rhythmMode" class="text-sm font-medium text-purple-900 font-semibold">
						🎵 Rhythm Mode (Crypt of the Necrodancer style)
					</label>
				</div>
				
				{#if rhythmMode}
					<div class="space-y-3 ml-6">
						<p class="text-xs text-purple-700">
							Players must move on the beat! Moves off-rhythm won't count.
						</p>
						
						<!-- Music vs Metronome Toggle -->
						<div class="flex items-center gap-3">
							<label class="flex items-center gap-2 cursor-pointer">
								<input
									type="radio"
									name="audioType"
									checked={rhythmUseMusic}
									onchange={() => rhythmUseMusic = true}
									disabled={loading}
									class="h-4 w-4 text-purple-600 focus:ring-purple-500"
								/>
								<span class="text-xs text-purple-900">🎵 Music</span>
							</label>
							<label class="flex items-center gap-2 cursor-pointer">
								<input
									type="radio"
									name="audioType"
									checked={!rhythmUseMusic}
									onchange={() => rhythmUseMusic = false}
									disabled={loading}
									class="h-4 w-4 text-purple-600 focus:ring-purple-500"
								/>
								<span class="text-xs text-purple-900">🔊 Metronome</span>
							</label>
						</div>
						
						<!-- Track Selection (only when music is enabled) -->
						{#if rhythmUseMusic}
							<div>
								<label for="rhythmTrack" class="block text-xs font-medium text-purple-900 mb-1">
									Music Track
								</label>
								<select
									id="rhythmTrack"
									bind:value={rhythmTrack}
									disabled={loading}
									class="w-full rounded-md border border-purple-300 bg-white px-3 py-2 text-sm text-purple-900 focus:border-purple-500 focus:ring-purple-500"
								>
									<option value="random">🎲 Random (different for each player)</option>
									{#each MUSIC_TRACKS as track, index}
										<option value={index.toString()}>
											🎵 {track.name} ({track.bpm} BPM)
										</option>
									{/each}
								</select>
								<p class="text-xs text-purple-600 mt-1">
									{#if rhythmTrack === 'random'}
										Each player gets a random track with its own BPM.
									{:else}
										All players play the same track ({MUSIC_TRACKS[parseInt(rhythmTrack)]?.bpm} BPM).
									{/if}
								</p>
							</div>
						{:else}
							<!-- BPM Field (only for metronome mode) -->
							<div>
								<label for="rhythmBPM" class="block text-xs font-medium text-purple-900 mb-1">
									Beats Per Minute (BPM): {rhythmBPM}
								</label>
								<input
									id="rhythmBPM"
									type="range"
									min="60"
									max="200"
									bind:value={rhythmBPM}
									disabled={loading}
									class="w-full h-2 bg-purple-200 rounded-lg appearance-none cursor-pointer"
								/>
								<div class="flex justify-between text-xs text-purple-600 mt-1">
									<span>60 (Slow)</span>
									<span>120 (Default)</span>
									<span>200 (Fast)</span>
								</div>
							</div>
						{/if}
						
						<!-- Tolerance Field -->
						<div>
							<label for="rhythmTolerance" class="block text-xs font-medium text-purple-900 mb-1">
								Timing Tolerance: {rhythmTolerance}ms
							</label>
							<input
								id="rhythmTolerance"
								type="range"
								min="50"
								max="500"
								step="10"
								bind:value={rhythmTolerance}
								disabled={loading}
								class="w-full h-2 bg-purple-200 rounded-lg appearance-none cursor-pointer"
							/>
							<div class="flex justify-between text-xs text-purple-600 mt-1">
								<span>50ms (Strict)</span>
								<span>150ms (Normal)</span>
								<span>500ms (Relaxed)</span>
							</div>
						</div>
					</div>
				{:else}
					<p class="text-xs text-purple-600 ml-6">
						Enable to require players to move on rhythm for an extra challenge!
					</p>
				{/if}
			</div>
		</div>

		<!-- Description Field -->
		<div class="form-field">
			<label for="description" class="block text-sm font-medium text-gray-700 mb-1">
				Description (optional)
			</label>
			<textarea
				id="description"
				bind:value={description}
				placeholder="Describe your tournament...

Examples:
• Weekly speedrun challenge - highest score wins!
• Practice tournament for beginners
• Prize: Top 3 get special Discord roles
• Rules: Must reach 2048 tile to qualify"
				class="w-full rounded-md border border-gray-300 p-3 text-sm min-h-[120px] resize-y focus:border-blue-500 focus:ring-1 focus:ring-blue-500"
				maxlength={2000}
				disabled={loading}
				rows={5}
			></textarea>
			<div class="mt-1 flex justify-between items-center">
				<div class="text-xs text-gray-500">
					{#if rhythmMode}
						<span class="text-purple-600">Rhythm settings will be added automatically.</span>
					{:else}
						<span>Add rules, prizes, or any details for participants.</span>
					{/if}
				</div>
				<span class="text-xs {description.length > 1800 ? 'text-amber-600' : 'text-gray-400'}">
					{description.length}/2000
				</span>
			</div>
		</div>

		<!-- Error Message -->
		{#if errorMessage}
			<div class="mb-4 rounded-md border border-red-400 bg-red-100 px-4 py-3 text-red-700">
				{errorMessage}
			</div>
		{/if}

		<!-- Submit Button -->
		<Button type="submit" variant="primary" {loading} class="w-full" disabled={loading}>
			{loading ? 'Submitting...' : 'Submit'}
		</Button>
	</div>
</form>

<style>
	:global(.game-font) {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
		-webkit-font-smoothing: antialiased;
	}

	form {
		border: 8px solid #bbada0;
		box-shadow:
			0 0 0 1px rgba(119, 110, 101, 0.08),
			0 8px 20px rgba(119, 110, 101, 0.2);
	}

	@media (max-width: 640px) {
		form {
			border-width: 4px;
			margin: 0 1rem;
		}
	}

	form::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-image:
			linear-gradient(#cdc1b4 1px, transparent 1px),
			linear-gradient(90deg, #cdc1b4 1px, transparent 1px);
		background-size: 20px 20px;
		background-position: -1px -1px;
		opacity: 0.05;
		pointer-events: none;
	}
</style>
