<script lang="ts">
    import Input from "../atoms/Input.svelte";
    import Button from "../atoms/Button.svelte";

    export let host: string;
    let totalRound = '';
    let eliminatedPerTrigger = '';
    let triggerInterval = '';
    let maxPlayer = '';
    let name = '';
    let loading = false;

    const handleSubmit = async () => {
        loading = true;
        try {
            await new Promise(resolve => setTimeout(resolve, 1000)); // Simulate API call
            console.log('Form submitted:', { totalRound, eliminatedPerTrigger, triggerInterval, maxPlayer, name });
        } finally {
            loading = false;
        }
    };
</script>

<form 
    on:submit|preventDefault={handleSubmit}
    class="max-w-md w-full mx-auto p-6 rounded-md bg-[#FAF8EF] shadow-md"
>
    <div class="space-y-6">
        <!-- Title -->
        <div class="text-center">
            <h2 class="text-[#776E65] text-2xl font-bold game-font">Multiplayer Game Settings</h2>
            <!-- <p class="text-[#776E65]/80 mt-2 game-font">Configure your game settings</p> -->
        </div>

        <!-- Name Field -->
        <div class="form-field">
            <Input
                id="name"
                label="Game Name"
                bind:value={name}
                placeholder="Enter game name"
                required
            />
        </div>

        <!-- Total Round Field -->
        <div class="form-field">
            <Input
                id="totalRound"
                label="Total Rounds"
                bind:value={totalRound}
                placeholder="Enter total rounds"
                required
                type="number"
            />
        </div>

        <!-- Eliminated Per Trigger Field -->
        <div class="form-field">
            <Input
                id="eliminatedPerTrigger"
                label="Eliminated Per Trigger"
                bind:value={eliminatedPerTrigger}
                placeholder="Enter number of eliminations per trigger"
                required
                type="number"
            />
        </div>

        <!-- Trigger Interval Field -->
        <div class="form-field">
            <Input
                id="triggerInterval"
                label="Trigger Interval (s)"
                bind:value={triggerInterval}
                placeholder="Enter trigger interval in seconds"
                required
                type="number"
            />
        </div>

        <!-- Max Player Field -->
        <div class="form-field">
            <Input
                id="maxPlayer"
                label="Max Players"
                bind:value={maxPlayer}
                placeholder="Enter maximum number of players"
                required
                type="number"
            />
        </div>

        <!-- Submit Button -->
        <Button
            type="submit"
            variant="primary"
            {loading}
            class="w-full"
        >
            {loading ? 'Creating...' : 'Create Game'}
        </Button>
    </div>
</form>

<style>
    /* 2048 font style */
    :global(.game-font) {
        font-family: "Clear Sans", "Helvetica Neue", Arial, sans-serif;
        -webkit-font-smoothing: antialiased;
    }

    /* Form container style */
    form {
        border: 8px solid #BBADA0;
        box-shadow: 
            0 0 0 1px rgba(119, 110, 101, 0.08),
            0 8px 20px rgba(119, 110, 101, 0.2);
    }

    /* Responsive padding */
    @media (max-width: 640px) {
        form {
            border-width: 4px;
            margin: 0 1rem;
        }
    }

    /* Optional: Add subtle grid pattern */
    form::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background-image: linear-gradient(#CDC1B4 1px, transparent 1px),
                          linear-gradient(90deg, #CDC1B4 1px, transparent 1px);
        background-size: 20px 20px;
        background-position: -1px -1px;
        opacity: 0.05;
        pointer-events: none;
    }
</style>