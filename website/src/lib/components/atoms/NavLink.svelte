<script lang="ts">
	interface Props {
		href: string;
		target: string;
		rel: string;
		children: () => any;
	}

	let { href, target = '_self', rel = '', children }: Props = $props();
</script>

<a
	class="nav-link text-shadow-md px-1 py-2 text-base font-bold text-red-500
	no-underline transition-colors duration-200 hover:text-white
	active:text-[#b3a9a1] lg:text-lg"
	{href}
	{target}
	{rel}
>
	{@render children()}
</a>
