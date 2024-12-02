<script lang="ts">
	// import Gamepad from 'lucide-svelte/icons/gamepad-2';
	// import UserRound from 'lucide-svelte/icons/user-round';

	interface Props {
		host?: string;
		totalBoards?: number;
		totalPlayers?: number;
	}

	let { host, totalBoards, totalPlayers }: Props = $props();
</script>

<div class="game-details relative text-xs md:text-base ms-3 md:ms-6">
	<div class="flex items-center justify-between gap-2 md:gap-4">
		<div class="flex items-center gap-2 md:gap-3 text-surface-200">
			[<span class="font-bold text-cyan-600">{host}</span>]
		</div>
        <div class="flex items-center gap-2">
			<span class="font-semibold text-surface-400">Boards:</span>
			<!-- <Gamepad size={18} color="#888" /> -->
			<span class="text-orange-600 game-details font-bold">{totalBoards}</span>
		</div>
        <div class="flex items-center gap-2">
            <span class="font-semibold text-surface-400">Players:</span>
            <!-- <UserRound size={18} color="#888" /> -->
            <span class="text-orange-600 game-details font-bold">{totalPlayers}</span>
        </div>
	</div>
</div>

<style>
	.game-details {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
	}
</style>
