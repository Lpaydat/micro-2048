<script lang="ts">
    import BrandLogo from "../atoms/BrandLogo.svelte";

    export let userRank: string;
    export let userScore: number;
</script>

<div class="flex items-center justify-between px-4 py-3 bg-white shadow-sm">
    <!-- Brand Section -->
    <div class="flex items-center">
        <BrandLogo />
    </div>
    
    <!-- Stats Section -->
    <div class="flex items-center space-x-4">
        <!-- Rank -->
        <div class="flex flex-col items-center">
            <span class="text-xs text-gray-500">Rank</span>
            <span class="font-medium text-gray-800">{userRank}</span>
        </div>
        
        <!-- Divider -->
        <div class="h-8 w-px bg-gray-200"></div>
        
        <!-- Score -->
        <div class="flex flex-col items-center">
            <span class="text-xs text-gray-500">Score</span>
            <span class="font-medium text-gray-800">{userScore}</span>
        </div>
    </div>
</div>