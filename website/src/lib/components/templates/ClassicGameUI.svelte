<script lang="ts">
	import BlockHashes from "../molecules/BlockHashes.svelte";
	import Game from "../organisms/Game.svelte";
	import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserSidebar from '../organisms/UserSidebar.svelte';
	import { userStore } from "$lib/stores/userStore";
	import { isHashesListVisible } from "$lib/stores/hashesStore";

    let canMakeMove = !!$userStore.username;
    $: chainId = $userStore.chainId;
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <UserSidebar />
    </svelte:fragment>

    <svelte:fragment slot="main">
        <div class="flex justify-center items-center h-full">
            <div class="w-full max-w-2xl pb-28 my-auto">
                <Game
                    player={$userStore.username ?? ''}
                    playerChainId={chainId as string}
                    canStartNewGame={!!$userStore.username}
                    showBestScore
                    canMakeMove={canMakeMove}
                />
            </div>
        </div>
        {#if $isHashesListVisible}
            <BlockHashes />
        {/if}
    </svelte:fragment>
</MainTemplate>
