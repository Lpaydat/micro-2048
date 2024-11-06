<script lang="ts">
	import GameModeCard from "../molecules/GameModeCard.svelte";
</script>

<div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4 max-w-4xl mx-auto">
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
</div>
