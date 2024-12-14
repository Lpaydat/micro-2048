<script lang="ts">
	import { page } from '$app/stores';
	import { TabGroup, Tab } from '@skeletonlabs/skeleton';
	import ListItem from '../molecules/LeaderboardItem.svelte';
	import type { PlayerStats, RoundResults } from '$lib/types/leaderboard';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import LeaderboardRankers from '../molecules/LeaderboardRankers.svelte';

	interface Props {
		currentRound?: number;
		player: string;
		currentPlayerScore?: number; // round score
		gameLeaderboard?: PlayerStats[];
		roundLeaderboard?: RoundResults;
		isFullScreen?: boolean;
	}

	let {
		currentRound = 1,
		player,
		currentPlayerScore = 0,
		gameLeaderboard = [],
		roundLeaderboard,
		isFullScreen = false
	}: Props = $props();

	const PLAYERS = gql`
		query Players($usernames: [String!]!) {
			players(usernames: $usernames) {
				username
				chainId
			}
		}
	`;

	// TODO: use elimination chainId
	const client = getContextClient();

	const players = $derived(
		queryStore({
			client,
			query: PLAYERS,
			variables: { usernames: gameLeaderboard?.map((p) => p.username) ?? [] }
		})
	);
	const currentUrl = $derived($page.url.pathname);
	const otherPlayersBoards = $derived(
		$players.data?.players.reduce(
			(acc: Record<string, string>, p: { username: string }) => {
				// Extract game ID and round from current URL
				const matches = currentUrl.match(/\/game\/(.+)-(\d+)-[^-]+$/);
				if (!matches) return acc;

				const [_, gameId, round] = matches;
				// Create new URL with player's username and chainId
				const boardId = `${gameId}-${round}-${p.username}`;
				const boardUrl = `/game/${boardId}`;

				acc[p.username] = boardUrl;
				return acc;
			},
			{} as Record<string, string>
		) ?? {}
	);

	const rlb = $derived(
		roundLeaderboard ?? {
			round: 0,
			players: [],
			eliminatedPlayers: []
		}
	);

	let activeTab = $state(1);

	const sortedGameLeaderboard = $derived(
		gameLeaderboard
			?.slice() // Create a shallow copy to avoid mutating the original array
			.sort((a, b) => b.score - a.score) // Sort by score in descending order
	);

	const combinedRoundLeaderboard = $derived(
		[...rlb.players, ...rlb.eliminatedPlayers]
			.map((p) => ({
				...p,
				isEliminated: rlb.eliminatedPlayers.includes(p),
				score: p.username === player ? currentPlayerScore : p.score
			}))
			.sort((a, b) => b.score - a.score)
	);

	const rankers = $derived(activeTab === 0 ? sortedGameLeaderboard : combinedRoundLeaderboard);

	const customClass = $derived(
		isFullScreen ? 'w-full h-full mt-4' : 'p-6 w-80 mt-6 max-h-full max-w-md mx-auto'
	);
</script>

<div class="text-center {customClass} overflow-hidden rounded-lg bg-[#FAF8EF] shadow-md">
	<header class="mb-4 flex flex-col items-center">
		<h1 class="mb-2 text-3xl font-bold text-[#776E65]">Leaderboard</h1>
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

	<LeaderboardRankers {rankers}>
		{#snippet item(rank, username, score, boardId, isEliminated)}
			<ListItem
				{rank}
				name={username}
				isCurrentPlayer={username === player}
				{score}
				{isEliminated}
				{boardId}
				boardUrl={otherPlayersBoards[username]}
			/>
		{/snippet}
	</LeaderboardRankers>
</div>

<style>
	div {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
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
