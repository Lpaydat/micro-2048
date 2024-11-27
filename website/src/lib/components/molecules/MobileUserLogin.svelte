<script lang="ts">
	import { getContextClient, queryStore, gql, mutationStore } from '@urql/svelte';
	import Input from '../atoms/Input.svelte';
	import Button from '../atoms/Button.svelte';
	import { hashPassword } from '$lib/utils/hashPassword';
	import { userStore } from '$lib/stores/userStore';
	import { getPlayerInfo } from '$lib/graphql/queries/getPlayerInfo';

	let username = '';
	let submittedUsername = '';
	let password = '';
	let passwordHash = '';
	let loading = false;
	let canLogin = false;
	let errorMessage = '';
	let showForm = false;

	const REGISTER_PLAYER = gql`
		mutation RegisterPlayer($username: String!, $passwordHash: String!) {
			registerPlayer(username: $username, passwordHash: $passwordHash)
		}
	`;

	const CHECK_PLAYER = gql`
		query CheckPlayer($username: String!, $passwordHash: String!) {
			checkPlayer(username: $username, passwordHash: $passwordHash)
		}
	`;

	const client = getContextClient();

	$: player = getPlayerInfo(client, submittedUsername);
	$: playerOnChain = queryStore({
		client,
		query: CHECK_PLAYER,
		variables: { username, passwordHash },
		requestPolicy: 'network-only'
	});

	const registerPlayer = async () => {
		mutationStore({
			client,
			query: REGISTER_PLAYER,
			variables: { username, passwordHash }
		});
	};

	const handleRegisterPlayer = async () => {
		registerPlayer();
		await new Promise((resolve) => setTimeout(resolve, 1000)); // Simulate API call
		username = '';
		password = '';
		player.reexecute({ requestPolicy: 'network-only' });
	};

	const checkPlayer = async () => {
		submittedUsername = username;
		playerOnChain.reexecute({ requestPolicy: 'network-only' });
	};

	const validateInput = (username: string, password: string) => {
		let errors = [];
		if (!username || !password) {
			errors.push('Please fill all fields');
		}
		return errors;
	};

	const handleSubmit = async () => {
		errorMessage = '';

		const errors = validateInput(username, password);
		if (errors.length > 0) {
			errorMessage = errors[0];
			return;
		}

		const encoder = new TextEncoder();
		passwordHash = await hashPassword(password, encoder.encode(username));
		loading = true;

		await checkPlayer();
	};

	$: {
		if (loading && username && !$playerOnChain.fetching) {
			try {
				const value = $playerOnChain.data?.checkPlayer;
				if (value === true) {
					canLogin = true;
					player.reexecute({ requestPolicy: 'network-only' });
				} else if (value === false) {
					errorMessage = 'Invalid credentials';
				} else {
					canLogin = true;
					handleRegisterPlayer();
				}
			} finally {
				loading = false;
			}
		}
	}

	$: if (!$player.fetching && $player.data?.player && canLogin) {
		localStorage.setItem('username', username);
		localStorage.setItem('passwordHash', passwordHash);
		localStorage.setItem('chainId', $player.data.player.chainId);
		localStorage.setItem('highestScore', $player.data.player.highestScore.toString());

		userStore.update((store) => ({
			...store,
			username: $player.data.player.username,
			chainId: $player.data.player.chainId,
			highestScore: $player.data.player.highestScore,
			...(passwordHash && { passwordHash })
		}));
		canLogin = false;
		showForm = false;
	}
</script>

<div class="relative z-20">
	<Button variant="outline" size="sm" on:click={() => (showForm = !showForm)}>Login</Button>

	{#if showForm}
		<div
			class="absolute right-0 top-12 w-72 rounded-md border-4 border-[#BBADA0] bg-[#FAF8EF] p-4 shadow-lg"
		>
			<form on:submit|preventDefault={handleSubmit} class="space-y-3">
				{#if errorMessage}
					<div class="error-message text-sm">{errorMessage}</div>
				{/if}

				<Input
					id="username"
					label="Username"
					bind:value={username}
					placeholder="Username"
					required
					size="sm"
				/>

				<Input
					id="password"
					type="password"
					label="Password"
					bind:value={password}
					placeholder="Password"
					required
					showToggle
					size="sm"
				/>

				<div class="flex justify-end gap-2">
					<Button variant="outline" size="sm" type="button" on:click={() => (showForm = false)}>
						Cancel
					</Button>
					<Button type="submit" variant="primary" size="sm" {loading}>
						{loading ? 'Loading...' : 'Login'}
					</Button>
				</div>
			</form>
		</div>
	{/if}
</div>

<style>
	.error-message {
		color: #d9534f;
		background-color: #f2dede;
		padding: 4px 8px;
		border-radius: 4px;
		text-align: center;
		margin-bottom: 0.5rem;
	}
</style>
