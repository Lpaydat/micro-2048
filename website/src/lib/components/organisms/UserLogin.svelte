<script lang="ts">
	import JoinForm from "../molecules/JoinForm.svelte";

    export let username: string | undefined;
</script>

<div>
    {#if username}
        <h1>Welcome, {username}!</h1>
    {:else}
        <div class="flex justify-center items-center h-full mx-auto">
            <JoinForm />
        </div>
    {/if}
</div>
