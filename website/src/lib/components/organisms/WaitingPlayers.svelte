<script lang="ts">
	import PlayerListItem from '../molecules/PlayerListItem.svelte';
	import { userStore } from '$lib/stores/userStore';

	export let players: Array<string> = [];

	// TODO: make sure to make the list not flicker when new players join

	// Convert simple string array to object array with default values
	$: playerObjects = players.map((name, index) => ({
		name,
		isHost: index === 0, // Assuming first player is host
		isYou: name === $userStore.username,
		joinedAt: new Date() // You might want to pass this as actual data
	}));
</script>

<div class="mx-auto mt-8 flex w-full flex-col gap-4 lg:max-w-4xl">
	<ul class="flex flex-col gap-4">
		{#if playerObjects.length > 0}
			{#each playerObjects as player}
				<li>
					<PlayerListItem
						playerName={player.name}
						isHost={player.isHost}
						isYou={player.isYou}
						joinedAt={player.joinedAt}
					/>
				</li>
			{/each}
		{:else}
			<li class="rounded-xl bg-white/90 p-8 text-center shadow-lg">
				<div
					class="mx-auto mb-4 flex h-20 w-20 items-center justify-center rounded-xl bg-[#edc53f]"
				>
					<i class="fas fa-user-group text-3xl text-white"></i>
				</div>
				<h3 class="mb-3 text-2xl font-bold text-[#edc53f]">NO PLAYERS YET</h3>
				<p class="inline-block rounded-full bg-[#bbada0] px-6 py-2 text-sm text-white">
					Waiting for other players to join...
				</p>
			</li>
		{/if}
	</ul>
</div>
