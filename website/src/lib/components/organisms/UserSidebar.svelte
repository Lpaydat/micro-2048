<script lang="ts">
    import Brand from "../molecules/Brand.svelte";
    import UserLogin from "./UserLogin.svelte";

    export let username: string | undefined;
</script>

<Brand />
<div class="flex items-center justify-center mt-24 w-full">
    <UserLogin bind:username />
</div>