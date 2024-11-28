<script lang="ts">
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import SinglePlayerLeaderboard from '../organisms/SinglePlayerLeaderboard.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';
</script>

<MainTemplate>
	{#snippet sidebar()}
		<UserSidebar />
	{/snippet}

	{#snippet main()}
		<SinglePlayerLeaderboard />
	{/snippet}
</MainTemplate>
