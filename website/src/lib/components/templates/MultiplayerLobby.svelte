<script lang="ts">
	import ActionButton from '../atoms/ActionButton.svelte';
	import Brand from "../molecules/Brand.svelte";
    import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserLogin from "../organisms/UserLogin.svelte";
	import WaitingGames from "../organisms/WaitingGames.svelte";

    export let username = 'lpaydat';

    const mockGames = [
        {
            name: "2048 Battle Royale",
            playerCount: 3,
            maxPlayers: 8,
            hostName: "GameMaster",
            createdAt: new Date(Date.now() - 5 * 60 * 1000), // 5 minutes ago
            totalRounds: 5,
            eliminatedPerRound: 2
        },
        {
            name: "Quick Elimination",
            playerCount: 6,
            maxPlayers: 6,
            hostName: "SpeedRunner",
            createdAt: new Date(Date.now() - 2 * 60 * 1000), // 2 minutes ago
            totalRounds: 3,
            eliminatedPerRound: 3
        },
        {
            name: "Casual Game",
            playerCount: 2,
            maxPlayers: 4,
            hostName: "Newbie123",
            createdAt: new Date(), // Just now
            totalRounds: 4,
            eliminatedPerRound: 1
        }
    ];
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <Brand />
        <UserLogin bind:username />
    </svelte:fragment>

    <svelte:fragment slot="main">
        <div class="h2 mb-4 font-bold flex items-center gap-4 bg-gradient-to-r from-warning-500 to-warning-600 text-white rounded-bl-lg p-4 shadow-md transition-all">
            <i class="fas fa-gamepad"></i>
            <span class="tracking-wider">ELIMINATION GAME</span>
            <ActionButton icon="plus" label="HOST GAME" />
        </div>
        <WaitingGames games={mockGames} />
    </svelte:fragment>
</MainTemplate>
