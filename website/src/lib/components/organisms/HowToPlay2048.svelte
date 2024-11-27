<script lang="ts">
	import HowToPlayModal from '../molecules/HowToPlayModal.svelte';

	const rules = [
		{
			text: 'Use arrow keys or swipe to move all tiles on the board.'
		},
		{
			text: 'When two tiles with the same number touch, they merge into one tile with double the value!'
		},
		{
			text: 'After each move, a new tile (2 or 4) appears on the board.'
		},
		{
			text: 'Try to create a tile with the number 2048 to win!'
		},
		{
			text: "Game over if you can't make any more moves."
		}
	];
</script>

<HowToPlayModal title="How to Play 2048" items={rules} />
