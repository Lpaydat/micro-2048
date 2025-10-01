<script lang="ts">
	import { fromZonedTime } from 'date-fns-tz';
	import Input from '../atoms/Input.svelte';
	import Button from '../atoms/Button.svelte';
	import { preventDefault } from '$lib/utils/preventDefault';
	import {
		createLeaderboard,
		type LeaderboardSettings
	} from '$lib/graphql/mutations/leaderboardAction.ts';
	import { getContextClient } from '@urql/svelte';
	import { getModalStore } from '@skeletonlabs/skeleton';

	// TODO: use main chainId
	const client = getContextClient();
	const modalStore = getModalStore();

	let name = $state('');
	let startTime = $state('');
	let endTime = $state('');
	let description = $state('');
	let loading = $state(false);
	let errorMessage = $state('');

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
				alert('Name cannot be empty.');
				return;
			}
			if (!startTime || !endTime) {
				alert('Start time and end time are required.');
				return;
			}
			if (new Date(startTime) >= new Date(endTime)) {
				alert('Start time must be before end time.');
				return;
			}

			const userTimeZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
			const settings: LeaderboardSettings = {
				name: eventName,
				description,
				startTime: fromZonedTime(new Date(startTime), userTimeZone).getTime().toString(),
				endTime: fromZonedTime(new Date(endTime), userTimeZone).getTime().toString(),
				shardNumber: 4, // Default to 4 shards
				baseTriggererCount: 2 // Default to 2 triggerers
			};

			console.log('Creating tournament with settings:', settings);
			console.log('User:', username);
			
			const result = createLeaderboard(client, settings);
			
			// Subscribe to the result to catch errors
			result.subscribe(($result) => {
				if ($result.error) {
					console.error('Tournament creation error:', $result.error);
					errorMessage = 'Failed to create tournament. Please check your credentials and try again.';
				} else if ($result.data) {
					console.log('Tournament created successfully:', $result.data);
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
				label="Start Time"
				bind:value={startTime}
				placeholder="Enter start time"
				required
				type="datetime-local"
				disabled={loading}
			/>
		</div>

		<!-- End Time Field -->
		<div class="form-field">
			<Input
				id="endTime"
				label="End Time"
				bind:value={endTime}
				placeholder="Enter end time"
				required
				type="datetime-local"
				disabled={loading}
			/>
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
