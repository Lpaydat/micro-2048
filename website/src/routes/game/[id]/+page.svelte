<script lang="ts">
    import GameUITemplate from "$lib/components/templates/GameUITemplate.svelte";
    import type { EliminationGameDetails } from '$lib/types/eliminationGame';

    export let data: EliminationGameDetails;

    let mockData = {
        name: "2048 Battle Royale",
        playerCount: 1,
        maxPlayers: 4,
        hostName: "lpaydat",
        createdAt: new Date(),
        totalRounds: 10,
        eliminatedPerTrigger: 1,
        triggerInterval: 10,
    }
</script>

<GameUITemplate data={mockData} isMultiplayer />