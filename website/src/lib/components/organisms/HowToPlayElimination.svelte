<script lang="ts">
	import HowToPlayModal from '../molecules/HowToPlayModal.svelte';

	const rules = [
		{
			text: 'Players compete in multiple rounds, trying to get the highest score.'
		},
		{
			text: 'Every few seconds (trigger), the players with lowest scores are eliminated.'
		},
		{
			text: 'Last player standing wins the round and everyone moves to next round.'
		},
		{
			text: 'Your total score adds up across all rounds.'
		},
		{
			text: "Click any player's name to watch their game live!"
		}
	];
</script>

<HowToPlayModal title="How to Play Elimination Mode" items={rules} />
