<script lang="ts">
    import { onMount } from 'svelte';
    import Main from "../molecules/Main.svelte";
    import MobileHeader from "../molecules/MobileHeader.svelte";
    import Sidebar from "../molecules/Sidebar.svelte";

    export let windowWidth = 0;

    onMount(() => {
        const updateWidth = () => (windowWidth = window.innerWidth);
        window.addEventListener('resize', updateWidth);
        updateWidth();
        return () => window.removeEventListener('resize', updateWidth);
    });

    $: isMobile = windowWidth <= 768;
    $: console.log('isMobile', isMobile);
</script>

<div class="flex h-screen overflow-hidden bg-[#23232b] bg-[url('/micro-carbon.png')] bg-repeat">
    {#if !isMobile}
        <Sidebar>
            <slot name="sidebar" />
        </Sidebar>
    {/if}

    <Main>
        <div class="flex flex-col flex-1">
            {#if isMobile}
                <div class="flex-none">
                    <MobileHeader>
                        <slot name="header" />
                    </MobileHeader>
                </div>
            {/if}

            <div class="flex-1 flex items-center lg:justify-start justify-center flex-col">
                <slot name="main" />
            </div>

            <div class="flex-none">
                <slot name="footer" />
            </div>
        </div>
    </Main>
</div>
