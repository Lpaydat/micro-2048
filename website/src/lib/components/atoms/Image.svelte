<script lang="ts">
	interface Props {
		src: string;
		alt: string;
	}

	let { src = '', alt = 'Image' }: Props = $props();
</script>

<img {src} {alt} />

<style>
	img {
		width: 100%;
		height: auto;
		max-width: 800px; /* Adjust this value to control the maximum size */
		display: block;
		margin: 0 auto;
	}
</style>
