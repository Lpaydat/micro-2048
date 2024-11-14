<script lang="ts">
    import Main from "../molecules/Main.svelte";
    import Sidebar from "../molecules/Sidebar.svelte";
</script>

<div class="flex h-screen overflow-hidden bg-[#23232b] bg-[url('/micro-carbon.png')] bg-repeat">
    <Sidebar>
        <slot name="sidebar" />
    </Sidebar>

    <Main>
        <slot name="main" />
    </Main>
</div>
