<script lang="ts">
    import MultiplayerLobby from "$lib/components/templates/MultiplayerLobby.svelte";

    export let username: string;
</script>

<MultiplayerLobby bind:username />