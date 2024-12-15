<script lang="ts">
	import { newGameBoard } from '$lib/game/newGameBoard';
	import { getBoardId } from '$lib/stores/boardId';
	import { userStore } from '$lib/stores/userStore';
	import GameModeCard from '../molecules/GameModeCard.svelte';
	// import GameModeDescription from "../molecules/GameModeDescription.svelte";
	// import classicImage from "$lib/assets/classic.webp";
	// import multiplayerImage from "$lib/assets/multiplayer.webp";

	// let imageUrl = classicImage;
	// let description = "Slide numbered tiles on a 4x4 grid to combine them and create a tile with the number 2048! Swipe to move tiles, and when two tiles with the same number collide, they merge into one with the sum of the two numbers. A new tile (2 or 4) appears after each move. The game ends when no moves remain, so plan carefully to reach 2048 or even higher!";
	// const multiplayerDescription = "Compete against others in this elimination-style version of 2048! Each player has a 4x4 grid, and the goal is to be the last one standing. Every few rounds, the player with the lowest score or tile value is eliminated. Keep combining tiles to stay ahead, and watch out for optional power-ups or penalties! The last player remaining wins!";

	// function handleMouseEnter(image: string, desc: string) {
	//     imageUrl = image;
	//     description = desc;
	// }

	// function handleMouseLeave() {
	//     imageUrl = classicImage;
	//     description = "Challenge yourself with the timeless 2048 puzzle. Strategically merge tiles to reach the elusive 2048 tile and beyond!";
	// }

	const lastBoardId = $derived(getBoardId());
	let boardId = $state('');

	$effect(() => {
		// Create a new board if not already created
		if (!lastBoardId && $userStore.username) {
			(async () => {
				boardId = await newGameBoard('');
			})();
		}
	});

	const lastBoardIdUrl = $derived(`/game?boardId=${lastBoardId || boardId}`);
</script>

<div class="mx-auto flex h-full max-w-4xl flex-col gap-4 p-4">
	<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
		<a href={lastBoardIdUrl}>
			<GameModeCard>
				{#snippet title()}
					<h2>Classic</h2>
				{/snippet}
				{#snippet description()}
					<p>The original 2048 experience. Combine matching numbers to reach 2048!</p>
				{/snippet}
			</GameModeCard>
		</a>
		<a href="/elimination">
			<GameModeCard>
				{#snippet title()}
					<h2>Multiplayer</h2>
				{/snippet}
				{#snippet description()}
					<p>Compete against other players in elimination mode. Last player standing wins!</p>
				{/snippet}
			</GameModeCard>
		</a>
		<a href="/events">
			<GameModeCard>
				{#snippet title()}
					<h2>Events</h2>
				{/snippet}
				{#snippet description()}
					<p>Check out the events and tournaments hosted by the community!</p>
				{/snippet}
			</GameModeCard>
		</a>
		<a href="/leaderboard">
			<GameModeCard>
				{#snippet title()}
					<h2>Leaderboard</h2>
				{/snippet}
				{#snippet description()}
					<p>Check out the highest single player scores and see where you rank!</p>
				{/snippet}
			</GameModeCard>
		</a>
	</div>
	<!-- <div class="flex justify-center items-center flex-grow">
        <GameModeDescription {imageUrl} {description} />
    </div> -->
</div>
