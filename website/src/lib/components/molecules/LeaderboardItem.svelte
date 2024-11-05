<script lang="ts">
    export let rank: number;
    export let name: string;
    export let score: number;
    export let isCurrentPlayer: boolean = false;
    export let isSelectedPlayer: boolean = false;
</script>

<li class="flex justify-between items-center odd:bg-[#EEE4DA] even:bg-[#F3F3F3] p-3 pl-2 rounded-sm shadow-sm relative border-l-4 border-transparent {isCurrentPlayer ? '!border-l-red-500 font-bold' : ''} {isSelectedPlayer ? '!border-l-orange-500' : ''}">
    <span class="font-bold text-surface-700 w-12 text-left">#{rank}</span>
    <span class="text-surface-800 flex-1 text-left ml-4">{name}</span>
    <span class="text-surface-600 w-16 text-right">{score}</span>
</li>

<style>
    li {
        border-bottom: 1px solid #BBADA0;
        transition: background-color 0.3s ease, transform 0.3s ease;
        position: relative;
    }
    li:hover {
        background-color: #D3C4B1;
        transform: translateY(-2px);
    }
    span {
        padding: 0 8px;
    }
</style>