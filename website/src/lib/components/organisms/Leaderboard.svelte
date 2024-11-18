<script lang="ts">
  import { page } from '$app/stores';
  import { TabGroup, Tab } from '@skeletonlabs/skeleton';
  import ListItem from '../molecules/LeaderboardItem.svelte'
	import type { PlayerStats, RoundResults } from '$lib/types/leaderboard';
	import { getContextClient, gql, queryStore } from '@urql/svelte';

  export let currentRound: number = 1;
  export let player: string;
  export let currentPlayerScore: number = 0; // round score
  export let gameLeaderboard: PlayerStats[] = [];
  export let roundLeaderboard: RoundResults | undefined;

  const PLAYERS = gql`
    query Players($usernames: [String!]!) {
      players(usernames: $usernames) {
        username
        chainId
      }
    }
  `;

  const client = getContextClient();

  $: players = queryStore({
    client,
    query: PLAYERS,
    variables: { usernames: gameLeaderboard?.map(p => p.username) ?? [] }
  });
  $: currentUrl = $page.url.pathname;
  $: otherPlayersBoards = $players.data?.players.reduce((acc: Record<string, string>, p: { username: string, chainId: string }) => {
    // Extract game ID and round from current URL
    const matches = currentUrl.match(/\/game\/(.+)-(\d+)-[^-]+-[^-]+$/);
    if (!matches) return acc;
    
    const [_, gameId, round] = matches;
    // Create new URL with player's username and chainId
    const boardUrl = `/game/${gameId}-${round}-${p.username}-${p.chainId}`;
    
    acc[p.username] = boardUrl;
    return acc;
  }, {} as Record<string, string>) ?? {};

  $: rlb = roundLeaderboard ?? {
    round: 0,
    players: [],
    eliminatedPlayers: []
  };

  let activeTab = 1;

  $: sortedGameLeaderboard = gameLeaderboard
    ?.slice() // Create a shallow copy to avoid mutating the original array
    .sort((a, b) => b.score - a.score) // Sort by score in descending order
    .map((player, index) => ({ ...player, rank: index + 1 })); // Add rank based on sorted position

  $: combinedRoundLeaderboard = [...rlb.players, ...rlb.eliminatedPlayers]
    .map(p => ({
      ...p,
      isEliminated: rlb.eliminatedPlayers.includes(p),
      score: p.username === player ? currentPlayerScore : p.score
    }))
    .sort((a, b) => b.score - a.score)
    .map((player, index) => ({ ...player, rank: index + 1 }));
</script>

<div class="text-center p-6 w-80 mt-6 max-h-full rounded-lg bg-[#FAF8EF] shadow-md max-w-md mx-auto">
  <header class="flex flex-col items-center mb-4">
    <h1 class="text-3xl font-bold text-[#776E65] mb-2">Leaderboard</h1>
    <TabGroup>
      <Tab class="hover:bg-transparent" bind:group={activeTab} name="tab1" value={0}>Game</Tab>
      <Tab class="hover:bg-transparent" bind:group={activeTab} name="tab2" value={1}>
        Round {currentRound}
      </Tab>
    </TabGroup>
    <!-- {#if selectedRound !== currentRound}
      <p class="text-sm text-gray-600 mt-2">Viewing Round {selectedRound}</p>
    {/if} -->
  </header>

  <div class="list-container overflow-y-auto overflow-x-hidden h-[calc(100%-3rem)]">
    {#if activeTab === 0}
      <ul class="list-none p-0 border-sm">
        {#each sortedGameLeaderboard as {rank, username, score}}
          <ListItem 
            {rank}
            name={username}
            isCurrentPlayer={username === player}
            {score}
            boardUrl={otherPlayersBoards[username]}
          />
        {/each}
      </ul>
    {:else if activeTab === 1}
      <ul class="list-none p-0 border-sm">
        {#each combinedRoundLeaderboard as {rank, username, score, isEliminated}}
          <ListItem
            {rank}
            name={username}
            isCurrentPlayer={username === player}
            {score}
            {isEliminated}
            boardUrl={otherPlayersBoards[username]}
          />
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

  .list-container {
    max-height: calc(100vh - 10rem); /* Adjust the height as needed */
    overflow-y: auto;
    scroll-snap-type: y mandatory; /* Enable vertical snapping */
  }
</style> 