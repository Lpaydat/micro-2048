<script lang="ts">
	export let href: string;
	export let target: string = "_self";
	export let rel: string = "";
</script>

<a
	class="nav-link font-bold text-base lg:text-lg no-underline transition-colors duration-200
	text-red-500 hover:text-white active:text-[#b3a9a1] text-shadow-md
	py-2 px-1"
	{href}
	{target}
	{rel}
>
	<slot />
</a>
