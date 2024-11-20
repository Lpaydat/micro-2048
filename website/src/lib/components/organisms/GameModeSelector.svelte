<script lang="ts">
	import GameModeCard from "../molecules/GameModeCard.svelte";
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
</script>

<div class="flex flex-col gap-4 p-4 max-w-4xl mx-auto h-full">
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <a href="/game">
            <GameModeCard>
                <svelte:fragment slot="title">
                    <h2>Classic</h2>
                </svelte:fragment>
                <svelte:fragment slot="description">
                    <p>The original 2048 experience. Combine matching numbers to reach 2048!</p>
                </svelte:fragment>
            </GameModeCard>
        </a>
        <a href="/elimination">
            <GameModeCard>
                <svelte:fragment slot="title">
                    <h2>Multiplayer</h2>
                </svelte:fragment>
                <svelte:fragment slot="description">
                    <p>Compete against other players in elimination mode. Last player standing wins!</p>
                </svelte:fragment>
            </GameModeCard>
        </a>
        <a href="/leaderboard">
            <GameModeCard>
                <svelte:fragment slot="title">
                    <h2>Leaderboard</h2>
                </svelte:fragment>
                <svelte:fragment slot="description">
                    <p>Check out the highest single player scores and see where you rank!</p>
                </svelte:fragment>
            </GameModeCard>
        </a>
    </div>
    <!-- <div class="flex justify-center items-center flex-grow">
        <GameModeDescription {imageUrl} {description} />
    </div> -->
</div>
