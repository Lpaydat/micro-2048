<script lang="ts">
	import NavLink from "../atoms/NavLink.svelte";
</script>

<div class="nav-container mt-5 me-5 z-10">
    <ul class="nav-list flex justify-end gap-8">
        <!-- Main Linera Sites -->
        <li>
            <NavLink href="https://linera.io" target="_blank" rel="noopener noreferrer">Linera.io</NavLink>
        </li>
        <li>
            <NavLink href="https://linera.dev" target="_blank" rel="noopener noreferrer">Documentation</NavLink>
        </li>
        
        <!-- Social/Community Links -->
        <li>
            <NavLink href="https://discord.gg/linera" target="_blank" rel="noopener noreferrer">Discord</NavLink>
        </li>
        <li>
            <NavLink href="https://t.me/linera" target="_blank" rel="noopener noreferrer">Telegram</NavLink>
        </li>
    </ul>
</div>
