<script lang="ts">
	import { onMount } from 'svelte';
	import { getContextClient, gql, queryStore } from '@urql/svelte';
	import MainTemplate from '../organisms/MainTemplate.svelte';
	import RankerLeaderboard from '../organisms/RankerLeaderboard.svelte';
	import UserSidebar from '../organisms/UserSidebar.svelte';

	interface Props {
		leaderboardId?: string;
	}

	let { leaderboardId = '' }: Props = $props();

	const LEADERBOARD = gql`
		query Leaderboard($leaderboardId: String!) {
			leaderboard(leaderboardId: $leaderboardId) {
				rankers {
					username
					score
					boardId
				}
			}
		}
	`;

	const client = getContextClient();

	const rankers = $derived(
		queryStore({
			client,
			query: LEADERBOARD,
			variables: { leaderboardId }
		})
	);

	// Sort the rankers by score in descending order
	const sortedRankers = $derived(
		$rankers.data?.leaderboard.rankers.slice().sort((a: any, b: any) => b.score - a.score)
	);

	onMount(() => {
		rankers.reexecute({ requestPolicy: 'network-only' });

		const interval = setInterval(() => {
			rankers.reexecute({ requestPolicy: 'network-only' });
		}, 5000);

		return () => clearInterval(interval);
	});
</script>

<MainTemplate>
	{#snippet sidebar()}
		<UserSidebar />
	{/snippet}

	{#snippet main()}
		<RankerLeaderboard rankers={sortedRankers} />
	{/snippet}
</MainTemplate>
