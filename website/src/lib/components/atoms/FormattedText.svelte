<script lang="ts">
	import { parseSimpleMarkdown, type FormattedLine } from '$lib/utils/simpleMarkdown';

	interface Props {
		text: string;
		class?: string;
		linkClass?: string;
	}

	let { text, class: className = '', linkClass = 'text-blue-600 hover:underline' }: Props = $props();

	const lines = $derived(parseSimpleMarkdown(text));
	
	// Check if there are any bullet points to render as a list
	const hasBullets = $derived(lines.some(line => line.isBullet));
</script>

{#if hasBullets}
	<div class={className}>
		{#each lines as line, i}
			{#if line.isBullet}
				<div class="flex items-start gap-2 mb-1">
					<span class="text-current opacity-60 select-none">•</span>
					<span>
						{#each line.segments as segment}
							{#if segment.type === 'bold'}
								<strong class="font-semibold">{segment.content}</strong>
							{:else if segment.type === 'italic'}
								<em>{segment.content}</em>
							{:else if segment.type === 'link'}
								<a 
									href={segment.href} 
									target="_blank" 
									rel="noopener noreferrer" 
									class={linkClass}
								>{segment.content}</a>
							{:else}
								{segment.content}
							{/if}
						{/each}
					</span>
				</div>
			{:else if line.segments.length > 0 && (line.segments.length > 1 || line.segments[0].content.trim())}
				<p class={i > 0 && !lines[i-1]?.isBullet ? 'mt-2' : ''}>
					{#each line.segments as segment}
						{#if segment.type === 'bold'}
							<strong class="font-semibold">{segment.content}</strong>
						{:else if segment.type === 'italic'}
							<em>{segment.content}</em>
						{:else if segment.type === 'link'}
							<a 
								href={segment.href} 
								target="_blank" 
								rel="noopener noreferrer" 
								class={linkClass}
							>{segment.content}</a>
						{:else}
							{segment.content}
						{/if}
					{/each}
				</p>
			{:else}
				<!-- Empty line - add spacing -->
				<div class="h-2"></div>
			{/if}
		{/each}
	</div>
{:else}
	<!-- No bullets, render as paragraphs -->
	<div class={className}>
		{#each lines as line, i}
			{#if line.segments.length > 0 && (line.segments.length > 1 || line.segments[0].content.trim())}
				<p class={i > 0 ? 'mt-1' : ''}>
					{#each line.segments as segment}
						{#if segment.type === 'bold'}
							<strong class="font-semibold">{segment.content}</strong>
						{:else if segment.type === 'italic'}
							<em>{segment.content}</em>
						{:else if segment.type === 'link'}
							<a 
								href={segment.href} 
								target="_blank" 
								rel="noopener noreferrer" 
								class={linkClass}
							>{segment.content}</a>
						{:else}
							{segment.content}
						{/if}
					{/each}
				</p>
			{:else if i > 0}
				<!-- Empty line between paragraphs -->
				<div class="h-1"></div>
			{/if}
		{/each}
	</div>
{/if}
