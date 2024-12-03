<script lang="ts">
	import { getContextClient, mutationStore, gql, queryStore } from '@urql/svelte';
	import Input from '../atoms/Input.svelte';
	import Button from '../atoms/Button.svelte';
	import { hashPassword } from '$lib/utils/hashPassword';
	import { userStore } from '$lib/stores/userStore';
	import { getPlayerInfo } from '$lib/graphql/queries/getPlayerInfo';
	import { getModalStore, type ModalSettings } from '@skeletonlabs/skeleton';
	import { preventDefault } from '$lib/utils/preventDefault';

	let username = $state('');
	let submittedUsername = $state('');
	let passwordHash = $state('');
	let password = $state('');
	let loading = $state(false);
	let canLogin = $state(false);

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

	const modalStore = getModalStore();
	const client = getContextClient();

	const howToPlayModal: ModalSettings = {
		type: 'component',
		component: 'how-to-play-2048'
	};

	const guestModeModal: ModalSettings = {
		type: 'alert',
		title: 'Oops! No Sneaking In! 🕵️‍♂️',
		body: 'Sorry, but our "Play as Guest" feature is still on vacation! Why not join us properly? Create an account - it\'s quick, free, and way more fun than trying to sneak in through the back door! 😉'
	};

	const player = $derived(getPlayerInfo(client, submittedUsername));
	const playerOnChain = $derived(
		queryStore({
			client,
			query: CHECK_PLAYER,
			variables: { username, passwordHash },
			requestPolicy: 'network-only'
		})
	);

	const registerPlayer = async () => {
		mutationStore({
			client,
			query: REGISTER_PLAYER,
			variables: { username, passwordHash }
		});
	};

	const checkPlayer = async () => {
		submittedUsername = username;
		playerOnChain.reexecute({ requestPolicy: 'network-only' });
	};

	const handleRegisterPlayer = async () => {
		registerPlayer();
		await new Promise((resolve) => setTimeout(resolve, 1000)); // Simulate API call
		username = '';
		password = '';
		player.reexecute({ requestPolicy: 'network-only' });
	};

	let errorMessage = $state('');

	const validateInput = (username: string, password: string) => {
		let errors = [];

		if (username.length < 3) {
			errors.push('Username too short.');
		}
		if (/\s/.test(username)) {
			errors.push('No spaces in username.');
		}
		if (password.length < 3) {
			errors.push('Password too short.');
		}
		if (/\s/.test(password)) {
			errors.push('No spaces in password.');
		}

		return errors;
	};

	const handleSubmit = async () => {
		// Reset error message
		errorMessage = '';

		// Validate inputs
		const playerUsername = username.trim().replace(/\s+/g, ' ');
		const playerPassword = password.trim();
		const errors = validateInput(playerUsername, playerPassword);
		if (errors.length > 0) {
			errorMessage = errors[0];
			return;
		}

		const encoder = new TextEncoder();
		passwordHash = await hashPassword(playerPassword, encoder.encode(playerUsername));
		loading = true;

		await checkPlayer();
	};

	$effect(() => {
		if (loading && username && !$playerOnChain.fetching) {
			try {
				const value = $playerOnChain.data?.checkPlayer;
				if (value === true) {
					canLogin = true;
					player.reexecute({ requestPolicy: 'network-only' });
				} else if (value === false) {
					errorMessage = 'Invalid password';
				} else if (value === null) {
					canLogin = true;
					handleRegisterPlayer();
				}
			} finally {
				loading = false;
			}
		}
	});

	$effect(() => {
		if (!$player.fetching && $player.data?.player && canLogin) {
			localStorage.setItem('username', username);
			localStorage.setItem('passwordHash', passwordHash);
			localStorage.setItem('chainId', $player.data.player.chainId);

			userStore.update((store) => ({
				...store,
				username: $player.data.player.username,
				chainId: $player.data.player.chainId,
				highestScore: $player.data.player.highestScore,
				...(passwordHash && { passwordHash })
			}));
			canLogin = false;
		}
	});
</script>

<form
	onsubmit={preventDefault(handleSubmit)}
	class="mx-auto w-full max-w-md rounded-md bg-[#FAF8EF] p-6 shadow-md"
>
	<div class="space-y-6">
		<!-- Title -->
		<div class="text-center">
			<h2 class="game-font text-2xl font-bold text-[#776E65]">Join Game</h2>
			<p class="game-font mt-2 text-[#776E65]/80">Sign in to save your progress</p>
		</div>

		<!-- Error Message -->
		{#if errorMessage}
			<div class="error-message shake-animation">
				{errorMessage}
			</div>
		{/if}

		<!-- Username Field -->
		<div class="form-field">
			<Input
				id="username"
				label="Username"
				bind:value={username}
				placeholder="Enter your username"
				required
			/>
		</div>

		<!-- Password Field -->
		<div class="form-field">
			<Input
				id="password"
				type="password"
				label="Password"
				bind:value={password}
				placeholder="Enter your password"
				required
				showToggle
			/>
		</div>

		<!-- Submit Button -->
		<Button type="submit" variant="primary" {loading} class="w-full">
			{loading ? 'Joining...' : 'Join Now'}
		</Button>

		<!-- Additional Actions -->
		<div class="flex items-center justify-between border-t border-[#CDC1B4] pt-4">
			<Button variant="outline" size="sm" onclick={() => modalStore.trigger(howToPlayModal)}>
				How to Play
			</Button>

			<Button variant="default" size="sm" onclick={() => modalStore.trigger(guestModeModal)}>
				Play as Guest
			</Button>
		</div>
	</div>
</form>

<style>
	/* 2048 font style */
	:global(.game-font) {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
		-webkit-font-smoothing: antialiased;
	}

	/* Simplified error message style */
	.error-message {
		color: #d9534f; /* Bootstrap danger color */
		background-color: #f2dede; /* Light red background */
		padding: 5px; /* Reduced padding */
		border-radius: 4px;
		text-align: center;
		font-size: 0.875rem; /* Smaller font size */
		margin-bottom: 0.5rem; /* Reduced margin */
	}

	/* Shake animation */
	@keyframes shake {
		0%,
		100% {
			transform: translateX(0);
		}
		10%,
		30%,
		50%,
		70%,
		90% {
			transform: translateX(-10px);
		}
		20%,
		40%,
		60%,
		80% {
			transform: translateX(10px);
		}
	}

	.shake-animation {
		animation: shake 0.5s;
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
