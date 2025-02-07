<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		onlyIcon?: boolean;
		icon?: Snippet;
		label?: string;
		disabled?: boolean;
		color?: 'default' | 'important' | 'warning' | 'danger' | 'disabled';
		hoverColor?: 'default' | 'important' | 'warning' | 'danger' | 'disabled';
		loading?: boolean;
		onclick?: (event: MouseEvent) => void;
	}

	let {
		onlyIcon = false,
		icon,
		label,
		disabled,
		color = 'default',
		hoverColor = 'default',
		loading,
		onclick
	}: Props = $props();

	const baseColorClasses = {
		default: 'bg-orange-600 text-white',
		important: 'text-orange-600 variant-outline-primary hover:text-white',
		warning: 'bg-warning-500',
		danger: 'bg-error-500',
		disabled: 'bg-surface-300-600-token text-surface-600-300-token cursor-not-allowed'
	};

	const hoverColorClasses = {
		default: 'hover:bg-orange-700 hover:text-white',
		important: 'hover:bg-orange-600',
		warning: 'hover:bg-warning-600',
		danger: 'hover:bg-error-800',
		disabled: ''
	};

	const colorClass = $derived(baseColorClasses[disabled ? 'disabled' : color]);
	const hoverClass = $derived(!disabled && !loading ? hoverColorClasses[hoverColor || color] : '');
	const loadingClass = $derived(loading ? 'cursor-not-allowed text-surface-400' : '');
</script>

<button
	class="flex items-center justify-center rounded-lg px-2 py-2 text-xs font-bold transition-all lg:px-4 lg:text-sm {colorClass} {hoverClass} {loading
		? 'cursor-not-allowed'
		: ''}"
	{onclick}
	disabled={disabled || loading}
>
	{#if loading}
		<div class="loading-spinner mr-2"></div>
	{:else if icon}
		{@render icon?.()}
	{/if}
	{#if !onlyIcon}
		<span class="tracking-wider {loadingClass}">{label}</span>
	{/if}
</button>

<style>
	.loading-spinner {
		border: 4px solid rgba(255, 255, 255, 0.3);
		border-top: 4px solid #fff;
		border-radius: 50%;
		width: 16px;
		height: 16px;
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		0% {
			transform: rotate(0deg);
		}
		100% {
			transform: rotate(360deg);
		}
	}

	button {
		border: 2px solid #fff;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
	}

	button:disabled {
		opacity: 0.5;
		border-color: rgba(255, 255, 255, 0.2);
		box-shadow: none;
	}
</style>
