<script lang="ts">
	import { userStore } from "$lib/stores/userStore";
	import JoinForm from "../molecules/JoinForm.svelte";
	import UserDetails from "./UserDetails.svelte";
</script>

<div class="w-full h-full flex items-end">
    {#if $userStore.username}
        <div class="w-full px-16 mx-auto">
            <UserDetails />
        </div>
    {:else}
        <div class="flex justify-center items-center mx-auto mb-16">
            <JoinForm />
        </div>
    {/if}
</div>
