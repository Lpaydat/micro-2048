<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		rankers: {
			username: string;
			score: number;
			boardId: string;
			isEliminated?: boolean;
			isEnded?: boolean;
		}[];
		item: Snippet<[number, string, number, string, boolean | undefined, boolean | undefined]>;
	}

	let { rankers, item }: Props = $props();
</script>

<div class="list-container h-full snap-y overflow-y-auto overflow-x-hidden">
	{#if rankers.length > 0}
		<ul class="border-sm list-none p-0">
			{#each rankers as { username, score, boardId, isEliminated, isEnded }, index}
				{@render item(index + 1, username, score, boardId, isEliminated, isEnded)}
			{/each}
		</ul>
	{:else}
		<div class="text-center text-sm text-[#776E65]">No players yet</div>
	{/if}
</div>
