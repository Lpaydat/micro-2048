<script lang="ts">
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import SinglePlayerLeaderboard from '../organisms/SinglePlayerLeaderboard.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
</script>

<MainTemplate>
	<svelte:fragment slot="sidebar">
		<UserSidebar />
	</svelte:fragment>

	<svelte:fragment slot="main">
		<SinglePlayerLeaderboard />
	</svelte:fragment>
</MainTemplate>
