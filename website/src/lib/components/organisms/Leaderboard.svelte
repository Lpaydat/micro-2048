<script lang="ts">
  import { popup, TabGroup, Tab } from '@skeletonlabs/skeleton';
  import type { PopupSettings } from '@skeletonlabs/skeleton';
  import ListItem from '../molecules/LeaderboardItem.svelte'
  import { ChevronDown } from 'lucide-svelte';

  export let currentRound: number = 1;

  let selectedRound = currentRound;
  let activeTab = 0;

  // Mock data - replace with real data
  const gameLeaderboard = [
    { rank: 1, name: 'lpaydat', score: 1000 },
    { rank: 2, name: 'Life Goes On', score: 800 },
    { rank: 3, name: 'RoyCrypto', score: 600 },
  ];

  const roundLeaderboard = [
    { rank: 1, name: 'Player 2', score: 300 },
    { rank: 2, name: 'Player 1', score: 250 },
    { rank: 3, name: 'Player 3', score: 200 },
  ];

  const blockHashes = [
    { block: 1, hash: '0x1234567890abcdef...' },
    { block: 2, hash: '0xabcdef1234567890...' },
  ];

  const popupRoundSelect: PopupSettings = {
    event: 'click',
    target: 'popupRoundSelect',
    placement: 'bottom'
  };
</script>

<div class="text-center p-6 w-80 rounded-lg bg-[#FAF8EF] shadow-md max-w-md mx-auto">
  <header class="flex flex-col items-center mb-4">
    <h1 class="text-3xl font-bold text-[#776E65] mb-2">Leaderboard</h1>
    <TabGroup>
      <Tab bind:group={activeTab} name="tab1" value={0}>Game</Tab>
      <Tab bind:group={activeTab} name="tab2" value={1}>
        Round {currentRound}
        <button 
          class="ml-2"
          use:popup={popupRoundSelect}
        >
          <ChevronDown size={18} />
        </button>
      </Tab>
      <Tab bind:group={activeTab} name="tab3" value={2}>Blocks</Tab>
    </TabGroup>
    {#if selectedRound !== currentRound}
      <p class="text-sm text-gray-600 mt-2">Viewing Round {selectedRound}</p>
    {/if}
  </header>

  <div class="overflow-y-auto h-[calc(100%-3rem)]">
    {#if activeTab === 0}
      <ul class="list-none p-0 border-sm">
        {#each gameLeaderboard as {rank, name, score}}
          <ListItem {rank} {name} {score} />
        {/each}
      </ul>
    {:else if activeTab === 1}
      <ul class="list-none p-0 border-sm">
        {#each roundLeaderboard as {rank, name, score}}
          <ListItem {rank} {name} {score} />
        {/each}
      </ul>
    {:else if activeTab === 2}
      <ul class="list-none p-0 border-sm">
        {#each blockHashes as {block, hash}}
          <li class="flex justify-between">
            <span>Block {block}</span>
            <span>{hash}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  div {
    font-family: "Clear Sans", "Helvetica Neue", Arial, sans-serif;
  }

  h1 {
    text-shadow: 2px 2px 4px rgba(0, 0, 0, 0.1);
  }

  .border-sm {
    border-radius: 6px !important;
  }
</style> 