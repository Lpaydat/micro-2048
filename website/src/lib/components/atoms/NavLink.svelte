<script lang="ts">
	export let href: string;
	export let target: string = '_self';
	export let rel: string = '';
</script>

<a
	class="nav-link text-shadow-md px-1 py-2 text-base font-bold text-red-500
	no-underline transition-colors duration-200 hover:text-white
	active:text-[#b3a9a1] lg:text-lg"
	{href}
	{target}
	{rel}
>
	<slot />
</a>
