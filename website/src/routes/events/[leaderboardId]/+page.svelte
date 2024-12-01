<script lang="ts">
	import { page } from '$app/stores';
	import Leaderboard from '$lib/components/templates/Leaderboard.svelte';

	const leaderboardId = $derived($page.params.leaderboardId);
</script>

<Leaderboard {leaderboardId} />

