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

	const client = getClient(mainChainId, true);
	console.log('EventLeaderboardForm - Using main chain ID:', mainChainId);
	console.log('EventLeaderboardForm - Client instance:', client);
	console.log('EventLeaderboardForm - Client URL:', (client as any).url);
	const modalStore = getModalStore();

	let name = $state('');
	let startTime = $state('');
	let endTime = $state('');
	let description = $state('');
	let shardNumber = $state(4);
	let baseTriggererCount = $state(2);
	let loading = $state(false);
	let errorMessage = $state('');

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

			// Set default times if not provided
			if (!startTime || !endTime) {
				setDefaultTimes();
			}

			if (new Date(startTime) >= new Date(endTime)) {
				errorMessage = 'Start time must be before end time.';
				return;
			}

			if (shardNumber < 1 || shardNumber > 20) {
				errorMessage = 'Shard number must be between 1 and 20.';
				return;
			}

			if (baseTriggererCount < 1 || baseTriggererCount > 10) {
				errorMessage = 'Base triggerer count must be between 1 and 10.';
				return;
			}

			const userTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
			const settings: LeaderboardSettings = {
				name: eventName,
				description: description || undefined,
				startTime: fromZonedTime(new Date(startTime), userTimeZone).getTime().toString(),
				endTime: fromZonedTime(new Date(endTime), userTimeZone).getTime().toString(),
				shardNumber: shardNumber,
				baseTriggererCount: baseTriggererCount
			};

			console.log('Creating tournament with settings:', settings);
			console.log('User:', username);
			console.log('Client URL:', (client as any).url);
			console.log('Expected main chain URL:', `http://localhost:8088/chains/${mainChainId}/applications/988d375aa1348035087f7a7bf7acec0931982df409f228c37781386505593a54`);
			
			const result = createLeaderboard(client, settings);
			
			if (!result) {
				errorMessage = 'Authentication failed. Please make sure you are logged in.';
				return;
			}
			
			// Subscribe to the result to catch errors
			result.subscribe(($result: any) => {
				console.log('Tournament creation result:', $result);
				if ($result.error) {
					console.error('Tournament creation error:', $result.error);
					console.error('Error details:', JSON.stringify($result.error, null, 2));
					errorMessage = 'Failed to create tournament. Please check your credentials and try again.';
				} else if ($result.data) {
					console.log('Tournament created successfully:', $result.data);
					alert('Tournament created successfully!');
					setTimeout(() => {
						modalStore.close();
					}, 1000);
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
	class="mx-auto w-full max-w-md rounded-md bg-[#FAF8EF] p-6 shadow-md"
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
			<Input
				id="startTime"
				label="Start Time (optional - defaults to 1 hour from now)"
				bind:value={startTime}
				placeholder="Enter start time"
				type="datetime-local"
				disabled={loading}
			/>
		</div>

		<!-- End Time Field -->
		<div class="form-field">
			<Input
				id="endTime"
				label="End Time (optional - defaults to 24 hours from now)"
				bind:value={endTime}
				placeholder="Enter end time"
				type="datetime-local"
				disabled={loading}
			/>
			<button
				type="button"
				onclick={setDefaultTimes}
				class="mt-2 text-xs text-blue-600 hover:text-blue-800 underline"
				disabled={loading}
			>
				Reset to default times
			</button>
		</div>

		<!-- Shard Number Field -->
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
			<p class="text-xs text-gray-600 mt-1">
				Number of parallel game chains. More shards = higher capacity but increased complexity.
			</p>
		</div>

		<!-- Base Triggerer Count Field -->
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
			<p class="text-xs text-gray-600 mt-1">
				Number of chains authorized to trigger score aggregation. Higher count = more redundancy.
			</p>
		</div>

		<!-- Description Field -->
		<div class="form-field">
			<textarea
				id="description"
				bind:value={description}
				placeholder="Enter description (max 200 chars)"
				class="w-full rounded-md border p-2"
				maxlength="200"
				disabled={loading}
			></textarea>
		</div>

		<!-- Error Message -->
		{#if errorMessage}
			<div class="rounded-md bg-red-100 border border-red-400 text-red-700 px-4 py-3 mb-4">
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
