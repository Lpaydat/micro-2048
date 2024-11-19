<script lang="ts">
    import { getModalStore } from '@skeletonlabs/skeleton';

    export let title = "How to Play";
    export let items: { text: string }[] = [];

    const modalStore = getModalStore();
</script>

<div class="bg-white rounded-lg shadow-xl w-full max-w-lg">
    <!-- Header -->
    <div class="p-3 sm:p-4 md:p-6 pb-2 sm:pb-3 md:pb-4 border-b">
        <h2 class="text-lg sm:text-xl md:text-2xl font-bold text-gray-800">{title}</h2>
    </div>

    <!-- Content -->
    <div class="p-3 sm:p-4 md:p-6 space-y-3 sm:space-y-4 md:space-y-6">
        <div class="space-y-2 sm:space-y-3 md:space-y-4">
            {#each items as item, i}
                <div class="flex items-start gap-2 sm:gap-3 md:gap-4">
                    <div class="w-6 sm:w-7 md:w-8 h-6 sm:h-7 md:h-8 rounded-full bg-warning-100 flex-shrink-0 flex items-center justify-center text-warning-700 font-medium text-sm sm:text-base">
                        {i + 1}
                    </div>
                    <p class="text-gray-700 text-sm sm:text-base md:text-lg pt-0.5 sm:pt-1">
                        {item.text}
                    </p>
                </div>
            {/each}
        </div>
    </div>

    <!-- Footer -->
    <div class="p-3 sm:p-4 md:p-6 pt-0">
        <button 
            class="w-full py-2 sm:py-2.5 md:py-3 bg-warning-500 text-white rounded-md hover:bg-warning-600 font-medium transition-colors text-sm sm:text-base md:text-lg"
            on:click={() => modalStore.close()}
        >
            Got it!
        </button>
    </div>
</div>
