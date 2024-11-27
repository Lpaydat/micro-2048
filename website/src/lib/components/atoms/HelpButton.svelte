<script lang="ts">
	interface Props {
		ariaLabel?: string;
		onclick?: () => void;
		children?: () => any;
	}

	let { ariaLabel, onclick, children }: Props = $props();
</script>

<button
	class="flex h-8 w-8 items-center justify-center rounded-md bg-white/20 transition-colors hover:bg-white/30"
	aria-label={ariaLabel}
	{onclick}
>
	{@render children?.()}
</button>
