<script lang="ts">
    import { onMount } from 'svelte';
    import Main from "../molecules/Main.svelte";
    import MobileHeader from "../molecules/MobileHeader.svelte";
    import Sidebar from "../molecules/Sidebar.svelte";
	import MobileUserDetails from '../molecules/MobileUserDetails.svelte';

    export let windowWidth = 0;
    export let mainCenter = false;
    export let overflowHidden = false;

    onMount(() => {
        const updateWidth = () => (windowWidth = window.innerWidth);
        window.addEventListener('resize', updateWidth);
        updateWidth();
        return () => window.removeEventListener('resize', updateWidth);
    });

    $: isMobile = windowWidth <= 768;
    const mainClass = mainCenter ? 'justify-center' : 'justify-start';
    const overflowClass = overflowHidden ? 'overflow-hidden' : '';
</script>

<div class="flex h-screen {overflowClass} bg-[#23232b] bg-[url('/micro-carbon.png')] bg-repeat">
    {#if !isMobile}
        <Sidebar>
            <slot name="sidebar" />
        </Sidebar>
    {/if}

    <Main>
        <div class="flex flex-col flex-1 {overflowClass}">
            {#if isMobile}
                <div class="flex-none">
                    <MobileHeader>
                        {#if $$slots.header}
                            <slot name="header" />
                        {:else}
                            <MobileUserDetails />
                        {/if}
                    </MobileHeader>
                </div>
            {/if}

            <div class="flex-1 flex items-center {mainClass} flex-col">
                <slot name="main" />
            </div>

            <div class="flex-none">
                <slot name="footer" />
            </div>
        </div>
    </Main>
</div>
