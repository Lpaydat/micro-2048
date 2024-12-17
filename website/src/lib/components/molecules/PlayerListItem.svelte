<script lang="ts">
	import { page } from '$app/stores';
	import Smile from 'lucide-svelte/icons/smile';
	import BaseListItem from './BaseListItem.svelte';

	interface Props {
		playerName: string;
		isHost: boolean;
		isYou: boolean;
	}

	let { playerName, isHost, isYou }: Props = $props();

	const competitivePhrases = [
		'Ready to dominate!',
		'Born to win!',
		'Challenge accepted!',
		'Bringing the heat 🔥',
		'Watch and learn!',
		'I will crush you all! 💪',
		'Prepare to be destroyed!',
		'Victory is my middle name',
		"You're all going down!",
		'Witness my power! ⚡',
		'Time for total domination',
		'Fear my wrath! 👑',
		'Your defeat is inevitable',
		'Bow before my skills!',
		'Resistance is futile',
		'Ultimate destroyer mode',
		'Here to end careers 😈',
		'The final boss has arrived',
		'Your nightmare begins now',
		'Prepare for annihilation',
		'Victory tastes sweet 🍯',
		'Time to assert dominance',
		'The throne is mine!',
		'Witness greatness mortals',
		'All shall fall before me',
		'The champion is here! 🏆',
		'Destruction incoming',
		'Your doom approaches',
		'Surrender now peasants',
		'Kneel before greatness',
		'Maximum carnage mode',
		'Time to feast on victory',
		'The apex predator 🦁',
		'Unleashing chaos mode',
		'Destruction guaranteed',
		'Your end is near',
		'Witness perfection',
		'Supreme victory incoming',
		'Absolute unit engaged',
		'Fear my final form! 🔥',
		'Destruction protocol active',
		'Prepare for obliteration',
		'Victory is inevitable',
		'The GOAT has arrived 🐐',
		"Time to show who's boss",
		'Ultimate power unleashed',
		'Witness my true power',
		'Victory shall be mine!',
		'Domination sequence start',
		'All shall remember this',
		'The legend has arrived'
	];

	function hashCode(str: string): number {
		let hash = 0;
		for (let i = 0; i < str.length; i++) {
			const char = str.charCodeAt(i);
			hash = (hash << 5) - hash + char;
			hash |= 0; // Convert to 32bit integer
		}
		return hash;
	}

	const gameId = $derived($page.params.gameId);
	const combinedString = $derived(`${gameId}-${playerName}`);
	const hashValue = $derived(hashCode(combinedString));
	const statusPhrase = $derived(
		competitivePhrases[Math.abs(hashValue) % competitivePhrases.length]
	);
</script>

<BaseListItem>
	{#snippet leftContent()}
		<div class="mb-1 flex items-center justify-between gap-2">
			<div class="flex items-center gap-2">
				<h3 class="text-lg font-semibold">{playerName}</h3>
				{#if isHost}
					<span class="rounded-full bg-orange-500 px-2 py-0.5 text-xs font-semibold text-white">
						HOST
					</span>
				{/if}
				{#if isYou}
					<Smile size={20} />
				{/if}
			</div>
			<div class="flex items-center gap-2 text-sm text-surface-700">
				<div class="h-4 w-4 rounded bg-warning-500"></div>
				<span>{statusPhrase}</span>
			</div>
		</div>
	{/snippet}
</BaseListItem>
