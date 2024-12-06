<script lang="ts">
	import { userStore } from '$lib/stores/userStore';
	import LeaderboardDetails from '../molecules/LeaderboardDetails.svelte';
	import ListItem from '../molecules/LeaderboardItem.svelte';
	import LeaderboardRankers from '../molecules/LeaderboardRankers.svelte';

	interface Props {
		isFullScreen?: boolean;
		leaderboardId?: string;
		name?: string;
		host?: string;
		startTime?: string;
		endTime?: string;
		totalBoards?: number;
		totalPlayers?: number;
		rankers?: { username: string; score: number; boardId: string }[];
	}

	let { isFullScreen, leaderboardId, rankers = [], ...rest }: Props = $props();

	const player = $derived($userStore.username);
	const customClass = isFullScreen ? 'w-full h-full' : 'p-6 w-80 max-h-full max-w-md mx-auto';
	const containerClass = isFullScreen ? 'mt-4' : 'mt-6';

	const getBoardUrl = (boardId: string) => {
		return leaderboardId
			? `/game/?boardId=${boardId}&leaderboardId=${leaderboardId}`
			: `/game/?boardId=${boardId}`;
	};
</script>

<div class="mx-auto flex max-w-sm flex-col gap-4 {containerClass} overflow-hidden">
	{#if leaderboardId}
		<div class="me-auto flex">
			<LeaderboardDetails {leaderboardId} {...rest} />
		</div>
	{/if}
	<div class="text-center {customClass} rounded-lg bg-[#FAF8EF] shadow-md">
		<header class="mb-4 flex flex-col items-center">
			<h1 class="mb-2 text-2xl font-bold text-[#776E65]">Leaderboard</h1>
		</header>

		<LeaderboardRankers {rankers}>
			{#snippet item(rank, username, score, boardId)}
				<ListItem
					{rank}
					name={username}
					isCurrentPlayer={username === player}
					{score}
					{boardId}
					boardUrl={getBoardUrl(boardId)}
				/>
			{/snippet}
		</LeaderboardRankers>
	</div>
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
