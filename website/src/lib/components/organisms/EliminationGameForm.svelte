<script lang="ts">
	import { getContextClient } from '@urql/svelte';
	import { getModalStore } from '@skeletonlabs/skeleton';

	import Input from '../atoms/Input.svelte';
	import Button from '../atoms/Button.svelte';
	import {
		createEliminationGame,
		type EliminationGameSettings
	} from '$lib/graphql/mutations/createEliminationGame';
	import { userStore } from '$lib/stores/userStore';
	import { preventDefault } from '$lib/utils/preventDefault';

	let totalRound = $state('5');
	let eliminatedPerTrigger = $state('2');
	let triggerInterval = $state('30');
	let maxPlayer = $state('10');
	let name = $state('');
	let loading = $state(false);

	const client = getContextClient();
	const modalStore = getModalStore();

	const handleSubmit = async () => {
		loading = true;
		try {
			// Validate inputs
			if (!name.trim()) {
				alert('Game name cannot be empty.');
				return;
			}
			if (
				parseInt(totalRound) < 1 ||
				parseInt(maxPlayer) < 1 ||
				parseInt(eliminatedPerTrigger) < 1 ||
				parseInt(triggerInterval) < 1
			) {
				alert('All numeric fields must be at least 1.');
				return;
			}
			if (!$userStore.username) {
				alert('Please login first.');
				return;
			}

			const settings: EliminationGameSettings = {
				gameName: name,
				totalRound: parseInt(totalRound),
				maxPlayers: parseInt(maxPlayer),
				eliminatedPerTrigger: parseInt(eliminatedPerTrigger),
				triggerIntervalSeconds: parseInt(triggerInterval),
				createdTime: Date.now().toString()
			};

			createEliminationGame(client, settings);
			await new Promise((resolve) => setTimeout(resolve, 1000)); // Simulate API call
			modalStore.close();
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
			<h2 class="game-font text-2xl font-bold text-[#776E65]">Multiplayer Game Settings</h2>
			<!-- <p class="text-[#776E65]/80 mt-2 game-font">Configure your game settings</p> -->
		</div>

		<!-- Name Field -->
		<div class="form-field">
			<Input id="name" label="Game Name" bind:value={name} placeholder="Enter game name" required />
		</div>

		<!-- Total Round Field -->
		<div class="form-field">
			<Input
				id="totalRound"
				label="Total Rounds"
				bind:value={totalRound}
				placeholder="Enter total rounds"
				required
				type="number"
				min="1"
				max="99"
			/>
		</div>

		<!-- Eliminated Per Trigger Field -->
		<div class="form-field">
			<Input
				id="eliminatedPerTrigger"
				label="Eliminated Per Trigger"
				bind:value={eliminatedPerTrigger}
				placeholder="Enter number of eliminations per trigger"
				required
				type="number"
				min="1"
				max="99"
			/>
		</div>

		<!-- Trigger Interval Field -->
		<div class="form-field">
			<Input
				id="triggerInterval"
				label="Trigger Interval (s)"
				bind:value={triggerInterval}
				placeholder="Enter trigger interval in seconds"
				required
				type="number"
				min="1"
				max="99"
			/>
		</div>

		<!-- Max Player Field -->
		<div class="form-field">
			<Input
				id="maxPlayer"
				label="Max Players"
				bind:value={maxPlayer}
				placeholder="Enter maximum number of players"
				required
				type="number"
				min="2"
				max="99"
			/>
		</div>

		<!-- Submit Button -->
		<Button type="submit" variant="primary" {loading} class="w-full">
			{loading ? 'Creating...' : 'Create Game'}
		</Button>
	</div>
</form>

<style>
	/* 2048 font style */
	:global(.game-font) {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
		-webkit-font-smoothing: antialiased;
	}

	/* Form container style */
	form {
		border: 8px solid #bbada0;
		box-shadow:
			0 0 0 1px rgba(119, 110, 101, 0.08),
			0 8px 20px rgba(119, 110, 101, 0.2);
	}

	/* Responsive padding */
	@media (max-width: 640px) {
		form {
			border-width: 4px;
			margin: 0 1rem;
		}
	}

	/* Optional: Add subtle grid pattern */
	form::before {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background-image: linear-gradient(#cdc1b4 1px, transparent 1px),
			linear-gradient(90deg, #cdc1b4 1px, transparent 1px);
		background-size: 20px 20px;
		background-position: -1px -1px;
		opacity: 0.05;
		pointer-events: none;
	}
</style>
