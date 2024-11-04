<script lang="ts">
    import Swords from 'lucide-svelte/icons/swords';
	import ActionButton from '../atoms/ActionButton.svelte';
	import PageHeader from "../molecules/PageHeader.svelte";
    import MainTemplate from "../organisms/MainTemplate.svelte";
	import UserSidebar from "../organisms/UserSidebar.svelte";
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
            eliminatedPerTrigger: 2
        },
        {
            name: "Quick Elimination",
            playerCount: 6,
            maxPlayers: 6,
            hostName: "SpeedRunner",
            createdAt: new Date(Date.now() - 2 * 60 * 1000), // 2 minutes ago
            totalRounds: 3,
            eliminatedPerTrigger: 3
        },
        {
            name: "Casual Game",
            playerCount: 2,
            maxPlayers: 4,
            hostName: "Newbie123",
            createdAt: new Date(), // Just now
            totalRounds: 4,
            eliminatedPerTrigger: 1
        }
    ];
</script>

<MainTemplate>
    <svelte:fragment slot="sidebar">
        <UserSidebar bind:username />
    </svelte:fragment>

    <svelte:fragment slot="main">
        <PageHeader title="ELIMINATION GAME">
            <svelte:fragment slot="icon">
                <Swords size={28} />
            </svelte:fragment>
            <svelte:fragment slot="actions">
                <ActionButton icon="plus" label="HOST GAME" />
            </svelte:fragment>
        </PageHeader>
        <WaitingGames games={mockGames} />
    </svelte:fragment>
</MainTemplate>
