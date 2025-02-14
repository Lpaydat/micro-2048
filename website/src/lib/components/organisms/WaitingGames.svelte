<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import type { EliminationGameDetails } from '$lib/types/eliminationGame';
	import GameListItem from '../molecules/GameListItem.svelte';
	import { userStore } from '$lib/stores/userStore';

	const client = getContextClient();

	let games: Array<EliminationGameDetails> = $state([]);

	const GET_WAITING_GAMES = gql`
		query GetWaitingGames {
			eliminationGames {
				gameId
				chainId
				gameName
				host
				players
				maxPlayers
				totalRounds
				triggerIntervalSeconds
				eliminatedPerTrigger
				createdTime
				status
			}
		}
	`;

	const waitingGames = $derived(
		queryStore({
			client,
			query: GET_WAITING_GAMES
		})
	);
	const rooms = $derived($waitingGames.data?.eliminationGames ?? []);

	let initialFetch = $state(true);
	$effect(() => {
		if (initialFetch && !$waitingGames.fetching) {
			initialFetch = false;
		}
	});

	let intervalId: NodeJS.Timeout;
	onMount(() => {
		waitingGames.reexecute({ requestPolicy: 'network-only' });

		intervalId = setInterval(() => {
			waitingGames.reexecute({ requestPolicy: 'network-only' });
			games = rooms.map((game: any) => {
				if (game.players.includes($userStore.username) && game.status === 'Waiting') {
					goto(`/elimination/${game.gameId}`);
				}

				return {
					...game,
					playerCount: game.players.length
				};
			});
		}, 1000);

		return () => {
			clearInterval(intervalId);
		};
	});
</script>

<ul class="mx-auto mt-8 flex h-full w-full max-w-4xl flex-col gap-4">
	{#if initialFetch}
		<li class="p-8 text-center">
			<div class="text-2xl font-bold text-[#776e65]">Loading...</div>
			<!-- Using 2048 game's yellow text color -->
		</li>
	{:else if games.length > 0}
		{#each games as game}
			<li>
				<a href={`/elimination/${game.gameId}`}>
					<GameListItem {...game} />
				</a>
			</li>
		{/each}
	{:else}
		<li class="rounded-xl bg-white/90 p-8 text-center shadow-lg">
			<div class="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-xl bg-[#edc53f]">
				<i class="fas fa-gamepad text-3xl text-white"></i>
			</div>
			<h3 class="mb-3 text-2xl font-bold text-[#d6b64c]">NO GAMES AVAILABLE</h3>
			<p class="inline-block rounded-full bg-[#bbada0] px-6 py-2 text-sm text-white">
				Create a new game or wait for others to host!
			</p>
		</li>
	{/if}
</ul>
