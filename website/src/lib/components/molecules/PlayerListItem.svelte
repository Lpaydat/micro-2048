<script lang="ts">
    import Smile from 'lucide-svelte/icons/smile';
    import BaseListItem from './BaseListItem.svelte';
    
    export let playerName: string;
    export let isHost: boolean = false;
    export let isYou: boolean = false;
    export let joinedAt: Date;

    // Format the joined time to show how long ago the player joined
    $: timeAgo = `${Math.floor((Date.now() - joinedAt.getTime()) / (1000 * 60))} minutes ago`;

    const competitivePhrases = [
        // Previous phrases
        "Ready to dominate!",
        "Born to win!",
        "Challenge accepted!",
        "Bringing the heat 🔥",
        "Watch and learn!",
        
        // New aggressive-fun phrases
        "I will crush you all! 💪",
        "Prepare to be destroyed!",
        "Victory is my middle name",
        "You're all going down!",
        "Witness my power! ⚡",
        "Time for total domination",
        "Fear my wrath! 👑",
        "Your defeat is inevitable",
        "Bow before my skills!",
        "Resistance is futile",
        "Ultimate destroyer mode",
        "Here to end careers 😈",
        "The final boss has arrived",
        "Your nightmare begins now",
        "Prepare for annihilation",
        "Victory tastes sweet 🍯",
        "Time to assert dominance",
        "The throne is mine!",
        "Witness greatness mortals",
        "All shall fall before me",
        "The champion is here! 🏆",
        "Destruction incoming",
        "Your doom approaches",
        "Surrender now peasants",
        "Kneel before greatness",
        "Maximum carnage mode",
        "Time to feast on victory",
        "The apex predator 🦁",
        "Unleashing chaos mode",
        "Destruction guaranteed",
        "Your end is near",
        "Witness perfection",
        "Supreme victory incoming",
        "Absolute unit engaged",
        "Fear my final form! 🔥",
        "Destruction protocol active",
        "Prepare for obliteration",
        "Victory is inevitable",
        "The GOAT has arrived 🐐",
        "Time to show who's boss",
        "Ultimate power unleashed",
        "Witness my true power",
        "Victory shall be mine!",
        "Domination sequence start",
        "All shall remember this",
        "The legend has arrived"
    ];

    $: statusPhrase = competitivePhrases[Math.floor(Math.random() * competitivePhrases.length)];
</script>

<BaseListItem>
    <div slot="left-content">
        <div class="flex items-center justify-between gap-2 mb-1">
            <div class="flex items-center gap-2">
                <h3 class="text-lg font-semibold">{playerName}</h3>
                {#if isHost}
                    <span class="text-xs font-semibold bg-orange-500 px-2 py-0.5 rounded-full text-white">
                        HOST
                    </span>
                {/if}
                {#if isYou}
                    <Smile size={20} />
                {/if}
            </div>
            <div class="flex items-center gap-2 text-sm text-surface-700">
                <div class="w-4 h-4 bg-warning-500 rounded"></div>
                <span>{statusPhrase}</span>
            </div>
        </div>
    </div>
</BaseListItem> 
