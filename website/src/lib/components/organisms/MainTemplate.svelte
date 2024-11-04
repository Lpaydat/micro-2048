<script lang="ts">
	import Main from "./Main.svelte";
    import Sidebar from "./Sidebar.svelte";
</script>

<div class="flex h-screen overflow-hidden bg-[#eee4da] bg-[url('/cardboard-texture.png')] bg-repeat">
    <Sidebar>
        <slot name="sidebar" />
    </Sidebar>

    <Main>
        <slot name="main" />
    </Main>
</div>
