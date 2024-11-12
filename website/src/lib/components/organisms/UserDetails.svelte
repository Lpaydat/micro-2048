<script lang="ts">
	import { userStore } from "$lib/stores/userStore";
    import Button from "../atoms/Button.svelte";

    const handleLogout = () => {
        $userStore.chainId = undefined;
        $userStore.username = undefined;
        $userStore.passwordHash = undefined;
    }
</script>

<div class="container w-full max-w-sm mx-auto p-6 rounded-md bg-[#FAF8EF] shadow-md border-[8px] border-[#BBADA0]">
    <div class="space-y-6">
        <!-- Title -->
        <div class="text-center">
            <h2 class="text-[#776E65] text-2xl font-bold game-font">Game Profile</h2>
        </div>

        <!-- User Info Box -->
        <div class="bg-[#bbada0] rounded-lg shadow-md p-4 w-full">
            <p class="text-[#eee4da] font-bold text-sm text-center game-font mb-1">
                LOGGED IN AS
            </p>
            <p class="text-[#f9f6f2] font-bold text-xl text-center game-font tracking-wide">
                {$userStore.username}
            </p>
        </div>
        
        <!-- Actions -->
        <div class="flex justify-between items-center pt-4 border-t border-[#CDC1B4]">
            <Button
                variant="outline"
                size="sm"
            >
                How to Play
            </Button>
            
            <Button
                variant="default"
                size="sm"
                on:click={handleLogout}
            >
                Logout
            </Button>
        </div>
    </div>
</div>

<style>
    :global(.game-font) {
        font-family: "Clear Sans", "Helvetica Neue", Arial, sans-serif;
        -webkit-font-smoothing: antialiased;
    }

    /* Responsive padding */
    @media (max-width: 640px) {
        div.container {
            border-width: 4px;
            margin: 0 1rem;
        }
    }
</style>
