<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
</script>

<div class="min-h-screen flex items-center justify-center bg-[#23232b] bg-[url('/micro-carbon.png')] bg-repeat">
  <div class="text-center p-8 rounded-2xl bg-[#23232b]/50 backdrop-blur-sm shadow-xl border border-[#3b3b41]">
    <!-- Game-style tile with error number -->
    <div class="mb-6 mx-auto w-32 h-32 flex items-center justify-center bg-[#edc22e] rounded-xl shadow-lg transform hover:scale-105 transition-transform">
      <span class="text-5xl font-bold text-white">{$page.status}</span>
    </div>

    <!-- Error message -->
    <h1 class="text-3xl font-bold text-white mb-4">
      {#if $page.status === 404}
        Page Not Found
      {:else}
        {$page.error?.message || 'An error occurred'}
      {/if}
    </h1>

    <!-- Custom message based on error -->
    <p class="text-gray-300 mb-8">
      {#if $page.status === 404}
        Oops! Looks like this tile slid off the board.
      {:else}
        Something went wrong while merging tiles.
      {/if}
    </p>

    <!-- Home button styled like a 2048 tile -->
    <button
      on:click={() => goto('/')}
      class="px-8 py-3 bg-[#edc22e] hover:bg-[#edcc60] text-white font-bold rounded-lg shadow-lg transform hover:scale-105 transition-all"
    >
      Back to Game
    </button>
  </div>
</div>