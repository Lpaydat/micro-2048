<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	interface Props extends Omit<HTMLInputAttributes, 'type' | 'placeholder' | 'size'> {
		label?: string | undefined;
		error?: string | undefined;
		showToggle?: boolean | undefined;
		type?: HTMLInputElement['type'];
		placeholder?: string;
		required?: boolean | undefined;
		size?: 'sm' | 'md' | 'lg';
	}

	let {
		value = $bindable(),
		label,
		error,
		type = 'text',
		placeholder,
		required,
		showToggle,
		size = 'md',
		...restProps
	}: Props = $props();

	let showPassword = $state(false);

	const togglePassword = () => {
		showPassword = !showPassword;
		type = showPassword ? 'text' : 'password';
	};

	// Helper function to get size-specific classes
	const getSizeClasses = (size: 'sm' | 'md' | 'lg') => {
		switch (size) {
			case 'sm':
				return 'px-3 py-2 text-sm';
			case 'lg':
				return 'px-5 py-4 text-xl';
			default:
				return 'px-4 py-3 text-lg';
		}
	};
</script>

<div class="form-field space-y-2">
	{#if label}
		<label
			for={restProps.id}
			class="game-font block font-bold text-gray-700 {size === 'sm' ? 'text-sm' : 'text-base'}"
		>
			{label}
			{#if required}
				<span class="text-orange-500">*</span>
			{/if}
		</label>
	{/if}

	<div class="relative">
		<input
			{...restProps}
			{type}
			{placeholder}
			bind:value
			class="
        game-font
        input
        w-full
        rounded-md
        border-2
        border-[#BBADA0]
        bg-[#FAF8EF]
        text-[#776E65]
        transition-all
        duration-200
        placeholder:text-[#CDC1B4]
        focus:border-[#EDC22E]
        focus:ring-2
        focus:ring-[#EDC22E]/30
        {getSizeClasses(size)}
        {restProps.class || ''}
      "
		/>

		{#if showToggle}
			<button
				type="button"
				class="absolute right-3 top-1/2 -translate-y-1/2 text-[#BBADA0] transition-colors hover:text-[#776E65] focus:outline-none"
				onclick={togglePassword}
				aria-label={showPassword ? 'Hide password' : 'Show password'}
			>
				{#if showPassword}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
						/>
					</svg>
				{:else}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="h-5 w-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
					>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
						/>
						<path
							stroke-linecap="round"
							stroke-linejoin="round"
							stroke-width="2"
							d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
						/>
					</svg>
				{/if}
			</button>
		{/if}
	</div>

	{#if error}
		<p class="game-font mt-1 text-sm text-[#F65E3B]">{error}</p>
	{/if}
</div>

<style>
	/* 2048 font style */
	:global(.game-font) {
		font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
		-webkit-font-smoothing: antialiased;
	}

	/* Smooth focus transition */
	input {
		outline: none;
		transition: all 0.3s ease;
	}

	/* Input hover effect */
	input:hover:not(:focus) {
		border-color: #cdc1b4;
	}

	/* Custom autofill style */
	input:-webkit-autofill,
	input:-webkit-autofill:hover,
	input:-webkit-autofill:focus {
		-webkit-text-fill-color: #776e65;
		-webkit-box-shadow: 0 0 0px 1000px #faf8ef inset;
		transition: background-color 5000s ease-in-out 0s;
	}

	/* Custom font for password input to reduce dot spacing */
	input[type='password'] {
		font-family: 'Courier New', Courier, monospace;
		letter-spacing: normal;
	}
</style>
