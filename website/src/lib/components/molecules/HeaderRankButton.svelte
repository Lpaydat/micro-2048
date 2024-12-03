<script lang="ts">
	interface Props {
		userRank: number;
		userScore: number;
		onclick: () => void;
	}

	const { userRank, userScore, onclick }: Props = $props();
</script>

<button
	type="button"
	{onclick}
	class="flex items-center justify-end rounded-lg bg-[#FAF8EF] p-2 shadow-md"
>
	<div class="flex items-center space-x-4 rounded-md bg-[#ed8d33] px-3 py-1">
		<!-- Rank -->
		<div class="flex flex-col items-center">
			<span class="text-xs uppercase text-[#EEE4DA]">Rank</span>
			<span class="font-bold text-white">#{userRank}</span>
		</div>

		<!-- Divider -->
		<div class="h-8 w-px bg-[#CDC1B4]"></div>

		<!-- Score -->
		<div class="flex flex-col items-center">
			<span class="text-xs uppercase text-[#EEE4DA]">Score</span>
			<span class="font-bold text-white">{userScore}</span>
		</div>
	</div>
</button>

<style>
	div {
		transition: all 0.3s ease;
	}
</style>
