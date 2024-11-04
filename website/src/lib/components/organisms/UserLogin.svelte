<script lang="ts">
	import JoinForm from "../molecules/JoinForm.svelte";
	import UserDetails from "./UserDetails.svelte";

    export let username: string | undefined;
</script>

<div class="w-full h-full flex items-end">
    {#if username}
        <div class="w-full px-16 mx-auto">
            <UserDetails username={username} />
        </div>
    {:else}
        <div class="flex justify-center items-center mx-auto mb-16">
            <JoinForm />
        </div>
    {/if}
</div>
