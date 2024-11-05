<script lang="ts">
  import { setContextClient } from '@urql/svelte';
  import { getClient } from '$lib/client';
  import Game from '$lib/components/organisms/Game.svelte';

  // const chainId = import.meta.env.VITE_CHAIN_ID;
  // const applicationId = import.meta.env.VITE_APPLICATION_ID;
  // const port = import.meta.env.VITE_PORT;

  const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
  const applicationId = '3326df61687858af29d28b71270a6ba8c259d7fb3a666ddef25f30038648d78cf933246bda05dd51d20395557cc6efa1b7fad0d5d0027d371059fc33978674bbe476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65c10000000000000000000000';
  const port = '8080';

  const client = getClient(chainId, applicationId, port);
  setContextClient(client);
</script>

<style>
  main {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    flex-direction: column;
    background-color: transparent;
  }

  h1 {
    font-size: 80px;
    margin: 0;
    padding-bottom: 20px;
    color: #776e65; /* Darker text color */
  }

  .game-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    max-width: 556px;
    margin: 0 auto;
    background-color: #bbada0; /* Game board background */
    padding: 20px;
    border-radius: 10px;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
  }

  .credits {
    font-size: 16px;
    font-weight: bold;
    color: #776e65;
  }

  .credits .heart {
    color: red; /* Make the heart emoji red */
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    max-width: 556px;
    margin: 0 auto;
  }

  .title {
    font-size: 80px;
    margin: 0;
    padding-bottom: 20px;
    color: #776e65; /* Darker text color */
  }
</style>

<main>
  <div class="header">
    <h1 class="title">2048</h1>
    <span class="credits">made with <span class="heart">❤️</span> lpaydat</span>
  </div>
  <div class="game-container">
    <Game />
  </div>
</main>
