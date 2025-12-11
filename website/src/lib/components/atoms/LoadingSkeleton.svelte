<!--
  LoadingSkeleton - Animated loading placeholder
  
  Provides visual feedback while content loads.
-->
<script lang="ts">
	interface Props {
		/** Width (CSS value) */
		width?: string;
		/** Height (CSS value) */
		height?: string;
		/** Border radius */
		rounded?: 'none' | 'sm' | 'md' | 'lg' | 'full';
		/** Number of lines (for text skeleton) */
		lines?: number;
		/** Line gap */
		gap?: string;
	}

	let {
		width = '100%',
		height = '1rem',
		rounded = 'md',
		lines = 1,
		gap = '0.5rem'
	}: Props = $props();

	const roundedClasses = {
		none: 'rounded-none',
		sm: 'rounded-sm',
		md: 'rounded-md',
		lg: 'rounded-lg',
		full: 'rounded-full'
	};
</script>

<div class="flex flex-col" style:gap>
	{#each Array(lines) as _, i}
		<div
			class="skeleton animate-pulse bg-gray-700/50 {roundedClasses[rounded]}"
			style:width={i === lines - 1 && lines > 1 ? '75%' : width}
			style:height
		></div>
	{/each}
</div>

<style>
	.skeleton {
		animation: shimmer 1.5s infinite;
		background: linear-gradient(
			90deg,
			rgb(55 65 81 / 0.5) 0%,
			rgb(75 85 99 / 0.5) 50%,
			rgb(55 65 81 / 0.5) 100%
		);
		background-size: 200% 100%;
	}

	@keyframes shimmer {
		0% {
			background-position: 200% 0;
		}
		100% {
			background-position: -200% 0;
		}
	}
</style>
