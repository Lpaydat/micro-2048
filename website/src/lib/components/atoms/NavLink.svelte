<script lang="ts">
	export let href: string;
	export let target: string = "_self";
	export let rel: string = "";
</script>

<a
	class="nav-link font-bold text-lg no-underline transition-colors duration-200
	text-[#776e65] hover:text-[#473d35] active:text-[#473d35]"
	{href}
	{target}
	{rel}
>
	<slot />
</a>
