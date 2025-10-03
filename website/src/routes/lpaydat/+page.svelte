<script lang="ts">
	import { goto } from '$app/navigation';
	import { toggleMod } from '$lib/graphql/mutations';
	import { getPlayerInfo } from '$lib/graphql/queries/getPlayerInfo';
	import { getContextClient } from '@urql/svelte';
	import { onMount, onDestroy } from 'svelte';
	import { getModalStore } from '@skeletonlabs/skeleton';

	let isReady = $state(false);
	let username = $state('lpaydat');
	let isToggling = $state(false);
	let lastOperation = $state<{ success: boolean; message: string; timestamp: number } | null>(null);
	let statusPollingInterval = $state<NodeJS.Timeout | null>(null);

	const client = getContextClient();
	const modalStore = getModalStore();

	const data = $derived(getPlayerInfo(client, username));

	const showFeedback = (success: boolean, message: string) => {
		lastOperation = { success, message, timestamp: Date.now() };
		// Clear feedback after 5 seconds
		setTimeout(() => {
			if (lastOperation?.timestamp === Date.now() - 5000) {
				lastOperation = null;
			}
		}, 5000);
	};

	const toggleModerator = async () => {
		const modUsername = username.trim();
		if (modUsername === '') {
			showFeedback(false, 'Please enter a username');
			return;
		}

		isToggling = true;
		try {
			toggleMod(client, modUsername);

			// Assume success and show feedback immediately
			const currentStatus = $data.data?.player?.isMod;
			const newStatus = !currentStatus;
			showFeedback(
				true,
				`Attempting to ${newStatus ? 'promote' : 'demote'} ${modUsername} ${newStatus ? 'to' : 'from'} moderator...`
			);

			// Start polling for status updates to confirm the change
			startStatusPolling(modUsername);
		} catch (error) {
			console.error('Toggle moderator error:', error);
			showFeedback(false, 'An error occurred while toggling moderator status');
		} finally {
			isToggling = false;
		}
	};

	const startStatusPolling = (targetUsername: string) => {
		// Clear any existing polling
		if (statusPollingInterval) {
			clearInterval(statusPollingInterval);
		}

		// Poll for 10 seconds to catch status updates
		let pollCount = 0;
		statusPollingInterval = setInterval(() => {
			pollCount++;
			data.reexecute({ variables: { username: targetUsername } });

			if (pollCount >= 10) {
				stopStatusPolling();
			}
		}, 1000);
	};

	const stopStatusPolling = () => {
		if (statusPollingInterval) {
			clearInterval(statusPollingInterval);
			statusPollingInterval = null;
		}
	};

	onMount(() => {
		const storedUsername = localStorage.getItem('username');
		if (storedUsername !== 'lpaydat') {
			goto('/error');
		}
		setTimeout(() => {
			isReady = true;
		}, 1000);
	});

	onDestroy(() => {
		stopStatusPolling();
	});
</script>

{#if isReady}
	<div
		class="flex min-h-screen w-full flex-col items-center justify-center gap-6 bg-surface-800 p-4"
	>
		<!-- Page Header -->
		<div class="text-center">
			<h1 class="mb-2 text-3xl font-bold text-white">Admin Panel</h1>
			<p class="max-w-md text-sm text-gray-300">
				Manage user moderator privileges. Enter a username below to toggle their moderator status.
			</p>
		</div>

		<!-- Input Section -->
		<div class="w-full max-w-md space-y-4">
			<div class="space-y-2">
				<label for="username" class="block text-sm font-medium text-gray-300">
					Target Username
				</label>
				<input
					id="username"
					type="text"
					class="w-full rounded-lg border border-surface-600 bg-surface-700 p-3 text-white placeholder-gray-400 focus:border-transparent focus:ring-2 focus:ring-primary-500"
					placeholder="Enter username to manage"
					bind:value={username}
				/>
			</div>

			<button
				type="button"
				class="variant-filled-primary btn w-full rounded-lg py-3 font-semibold disabled:cursor-not-allowed disabled:opacity-50"
				onclick={toggleModerator}
				disabled={isToggling}
			>
				{#if isToggling}
					<span class="flex items-center gap-2">
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-white border-t-transparent"
						></div>
						Processing...
					</span>
				{:else}
					Toggle Moderator Status
				{/if}
			</button>
		</div>

		<!-- Feedback Messages -->
		{#if lastOperation}
			<div class="w-full max-w-md">
				<div
					class="rounded-lg p-4 {lastOperation.success
						? 'border border-green-500/50 bg-green-900/50 text-green-200'
						: 'border border-red-500/50 bg-red-900/50 text-red-200'}"
				>
					<div class="flex items-center gap-2">
						{#if lastOperation.success}
							<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
								<path
									fill-rule="evenodd"
									d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
									clip-rule="evenodd"
								></path>
							</svg>
						{:else}
							<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
								<path
									fill-rule="evenodd"
									d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z"
									clip-rule="evenodd"
								></path>
							</svg>
						{/if}
						<span class="font-medium">{lastOperation.message}</span>
					</div>
				</div>
			</div>
		{/if}

		<!-- User Status Display -->
		<div class="w-full max-w-md">
			<div class="rounded-lg border border-surface-600 bg-surface-700 p-6">
				<h3 class="mb-4 text-center text-lg font-semibold text-white">User Status</h3>

				{#if $data.fetching}
					<div class="flex items-center justify-center gap-2 text-gray-400">
						<div
							class="h-4 w-4 animate-spin rounded-full border-2 border-gray-400 border-t-transparent"
						></div>
						<span>Loading user info...</span>
					</div>
				{:else if $data.data?.player}
					<div class="space-y-3">
						<div class="flex items-center justify-between">
							<span class="text-gray-300">Username:</span>
							<span class="font-medium text-white">{$data.data.player.username}</span>
						</div>

						<div class="flex items-center justify-between">
							<span class="text-gray-300">Role:</span>
							<span class="flex items-center gap-2">
								{#if $data.data.player.isMod}
									<span
										class="inline-flex items-center rounded-full bg-orange-900 px-2.5 py-0.5 text-xs font-medium text-orange-200"
									>
										<svg class="mr-1 h-3 w-3" fill="currentColor" viewBox="0 0 20 20">
											<path
												fill-rule="evenodd"
												d="M11.3 1.046A1 1 0 0112 2v5h4a1 1 0 01.82 1.573l-7 10A1 1 0 018 18v-5H4a1 1 0 01-.82-1.573l7-10a1 1 0 011.12-.38z"
												clip-rule="evenodd"
											></path>
										</svg>
										Moderator
									</span>
								{:else}
									<span
										class="inline-flex items-center rounded-full bg-gray-700 px-2.5 py-0.5 text-xs font-medium text-gray-300"
									>
										User
									</span>
								{/if}
							</span>
						</div>

						{#if statusPollingInterval}
							<div class="mt-2 text-center text-xs text-blue-400">Updating status...</div>
						{/if}
					</div>
				{:else}
					<div class="py-4 text-center text-gray-400">
						<svg
							class="mx-auto mb-2 h-12 w-12 opacity-50"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								stroke-width="2"
								d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
							></path>
						</svg>
						<p>User not found</p>
						<p class="mt-1 text-xs">Enter a valid username above</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
