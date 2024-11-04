<script lang="ts">
	import Brand from "../organisms/Brand.svelte";
    import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserLogin from "../organisms/UserLogin.svelte";
	import WaitingGames from "../organisms/WaitingGames.svelte";
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <Brand />
        <UserLogin />
    </svelte:fragment>

    <svelte:fragment slot="main">
        <div>Elimination Game</div>
        <WaitingGames />
    </svelte:fragment>
</MainTemplate>
