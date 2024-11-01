<script lang="ts">
  const chainId = import.meta.env.VITE_CHAIN_ID;
  const applicationId = import.meta.env.VITE_APPLICATION_ID;
  const port = import.meta.env.VITE_PORT;

  import { setContextClient } from '@urql/svelte';
  import { getClient } from './lib/client';
  import Game from './components/Game.svelte';

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
