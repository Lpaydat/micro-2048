<script lang="ts">
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface $$Props extends HTMLButtonAttributes {
    variant?: 'default' | 'primary' | 'destructive' | 'outline';
    size?: 'sm' | 'md' | 'lg';
    loading?: boolean;
    disabled?: boolean;
  }

  export let variant: $$Props['variant'] = 'default';
  export let size: $$Props['size'] = 'md';
  export let loading = false;
  export let disabled = false;

  const variantStyles = {
    default: 'bg-[#8F7A66] hover:bg-[#806A56] text-[#F9F6F2]',
    primary: 'bg-[#EDC22E] hover:bg-[#EDC53F] text-[#F9F6F2]',
    destructive: 'bg-[#F65E3B] hover:bg-[#F67C5F] text-[#F9F6F2]',
    outline: 'bg-transparent hover:bg-[#CDC1B4] text-[#776E65] border-2 border-[#776E65]'
  };

  const sizeStyles = {
    sm: 'px-3 py-2 text-sm',
    md: 'px-5 py-3 text-base',
    lg: 'px-6 py-4 text-lg'
  };

  $: classes = `
    relative
    game-font
    font-bold
    rounded-[3px]
    transition-all
    duration-200
    ${variantStyles[variant as keyof typeof variantStyles]}
    ${sizeStyles[size as keyof typeof sizeStyles]}
    ${disabled ? 'opacity-50 cursor-not-allowed' : 'transform hover:-translate-y-[1px]'}
    focus:outline-none
    focus:ring-2
    focus:ring-[#EDC22E]/50
    active:translate-y-[1px]
    ${$$props.class || ''}
  `;
</script>

<button
  {...$$props}
  {disabled}
  class={classes}
  on:click
  on:mouseover
  on:mouseenter
  on:mouseleave
  on:focus
  on:blur
>
  <div class="relative flex items-center justify-center gap-2">
    {#if loading}
      <div class="loading-dots"></div>
    {/if}
    <slot />
  </div>
</button>

<style>
  /* 2048 font style */
  :global(.game-font) {
    font-family: "Clear Sans", "Helvetica Neue", Arial, sans-serif;
    -webkit-font-smoothing: antialiased;
  }

  /* Loading animation */
  .loading-dots {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 20px;
  }

  .loading-dots::after {
    content: "";
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background-color: currentColor;
    animation: dots 1s infinite;
  }

  .loading-dots::before {
    content: "";
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background-color: currentColor;
    animation: dots 1s infinite;
    animation-delay: 0.5s;
  }

  @keyframes dots {
    0%, 100% {
      opacity: 0.2;
      transform: scale(0.8);
    }
    50% {
      opacity: 1;
      transform: scale(1.2);
    }
  }

  /* Button shadow effect */
  button {
    box-shadow: 0 4px 0 rgba(0, 0, 0, 0.1);
  }

  button:active {
    box-shadow: 0 2px 0 rgba(0, 0, 0, 0.1);
  }

  /* Disabled state */
  button:disabled {
    box-shadow: none;
  }
</style>
