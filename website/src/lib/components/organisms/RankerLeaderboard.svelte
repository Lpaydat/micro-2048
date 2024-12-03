<script lang="ts">
	import LeaderboardDetails from '../molecules/LeaderboardDetails.svelte';
	import RankerCard from '../molecules/RankerCard.svelte';

	interface Props {
		leaderboardId?: string;
		name?: string;
		host?: string;
		startTime?: string;
		endTime?: string;
		totalBoards?: number;
		totalPlayers?: number;
		rankers?: {
			username: string;
			score: number;
			boardId: string;
		}[];
	}

	let { rankers = [], leaderboardId, ...rest }: Props = $props();
</script>

<div
	class="mx-auto mt-4 flex h-[calc(100vh-8rem)] w-full max-w-4xl flex-col overflow-hidden md:mt-8"
>
	<div class="flex gap-3 md:gap-6">
		<h1
			class="md:ms-none mb-4 ms-2 text-center text-2xl font-extrabold text-gray-100 md:mb-6 md:text-4xl"
		>
			Leaderboard
		</h1>
		{#if leaderboardId}
			<LeaderboardDetails {...rest} />
		{/if}
	</div>
	<div class="flex-1 overflow-hidden bg-black/40 px-2 py-6 shadow-xl lg:rounded-lg lg:p-6">
		{#if (rankers?.length ?? 0) > 0}
			<div class="flex h-full flex-col overflow-visible">
				<div
					class="flex items-center justify-between border-b border-gray-700 px-4 pb-2 text-xs font-semibold text-gray-400 lg:text-base"
				>
					<span class="w-1/12 text-left">#</span>
					<span class="w-5/12">Player</span>
					<span class="w-3/12">Board ID</span>
					<span class="w-3/12 text-right">Score</span>
				</div>
				<div class="flex-1 space-y-4 overflow-y-auto overflow-x-hidden pt-4">
					{#each rankers as player, index}
						<RankerCard {player} rank={index + 1} />
					{/each}
				</div>
			</div>
		{:else}
			<div class="text-md flex h-full flex-col items-center justify-center md:text-lg">
				<p class="text-center font-semibold text-gray-400">No players yet.</p>
				<p class="text-center font-semibold text-gray-400">
					Be the first to join and claim the top spot!
				</p>
			</div>
		{/if}
	</div>
</div>
