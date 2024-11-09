<script lang="ts">
    export let rank: number;
    export let name: string;
    export let score: number;
    export let isEliminated: boolean = false;
    export let isCurrentPlayer: boolean = false;
    export let isSelectedPlayer: boolean = false;

    $: color = isEliminated ? 'bg-[#F3F3F3]' : 'bg-[#EEE4DA]';
    $: currentPlayerStyle = isCurrentPlayer ? '!border-l-blue-500 font-bold bg-[#FFD700]' : '';
    $: selectedPlayerStyle = isSelectedPlayer ? '!border-l-orange-500' : '';
</script>

<button type="button" on:click class="flex justify-between snap-start items-center p-3 pl-2 w-full {color} rounded-sm shadow-md relative border-l-4 border-transparent {currentPlayerStyle} {selectedPlayerStyle}">
    <span class="font-bold text-surface-700 w-12 text-left">#{rank}</span>
    <span class="text-surface-800 flex-1 text-left ml-4">{name}</span>
    <span class="text-surface-600 w-16 text-right">{score}</span>
</button>

<style>
    button {
        border-bottom: 1px solid #BBADA0;
        transition: background-color 0.3s ease, transform 0.3s ease;
        position: relative;
    }
    button:hover {
        background-color: #D3C4B1;
    }
    span {
        padding: 0 8px;
    }
    /* .animate-pulse {
        animation: pulse 2s infinite;
    } */
    /* @keyframes pulse {
        0%, 100% {
            transform: scale(1);
        }
        50% {
            transform: scale(1.05);
        }
    } */
</style>