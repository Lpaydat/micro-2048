<script lang="ts">
	import { getContextClient, mutationStore, gql } from "@urql/svelte";
  import Input from "../atoms/Input.svelte";
  import Button from "../atoms/Button.svelte";
	import { hashPassword } from "$lib/utils/hashPassword";
	import { userStore } from "$lib/stores/userStore";
	import { getPlayerInfo } from "$lib/graphql/queries/getPlayerInfo";

  let username = '';
  let submittedUsername = '';
  let passwordHash = '';
  let password = '';
  let loading = false;

  const REGISTER_PLAYER = gql`
    mutation RegisterPlayer($username: String!, $passwordHash: String!) {
      registerPlayer(username: $username, passwordHash: $passwordHash)
    }
  `;

  const client = getContextClient();

  $: player = getPlayerInfo(client, submittedUsername)

  // TODO: make it loginable and check if user already exists
  $: if (!$player.fetching && $player.data?.player) {
    userStore.update(store => ({
      ...store,
      username: $player.data.player.username,
      chainId: $player.data.player.chainId
    }));
  }

  const registerPlayer = async () => {
    passwordHash = await hashPassword(password);
    mutationStore({
      client,
      query: REGISTER_PLAYER,
      variables: { username, passwordHash }
    })
  };

  const handleSubmit = async () => {
    loading = true;
    submittedUsername = username;
    try {
      registerPlayer()
      await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call
      // localStorage.setItem('username', username);
      username = '';
      password = '';
      player.reexecute({ requestPolicy: 'network-only' });
    } finally {
      loading = false;
    }
  };
</script>

<form 
  on:submit|preventDefault={handleSubmit}
  class="max-w-md w-full mx-auto p-6 rounded-md bg-[#FAF8EF] shadow-md"
>
  <div class="space-y-6">
    <!-- Title -->
    <div class="text-center">
      <h2 class="text-[#776E65] text-2xl font-bold game-font">Join Game</h2>
      <p class="text-[#776E65]/80 mt-2 game-font">Sign in to save your progress</p>
    </div>

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
    <Button
      type="submit"
      variant="primary"
      {loading}
      class="w-full"
    >
      {loading ? 'Joining...' : 'Join Now'}
    </Button>

    <!-- Additional Actions -->
    <div class="flex justify-between items-center pt-4 border-t border-[#CDC1B4]">
      <Button
        variant="outline"
        size="sm"
      >
        How to Play
      </Button>
      
      <Button
        variant="default"
        size="sm"
      >
        Play as Guest
      </Button>
    </div>
  </div>
</form>

<style>
  /* 2048 font style */
  :global(.game-font) {
    font-family: "Clear Sans", "Helvetica Neue", Arial, sans-serif;
    -webkit-font-smoothing: antialiased;
  }

  /* Form container style */
  form {
    border: 8px solid #BBADA0;
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
    background-image: linear-gradient(#CDC1B4 1px, transparent 1px),
                      linear-gradient(90deg, #CDC1B4 1px, transparent 1px);
    background-size: 20px 20px;
    background-position: -1px -1px;
    opacity: 0.05;
    pointer-events: none;
  }
</style>
