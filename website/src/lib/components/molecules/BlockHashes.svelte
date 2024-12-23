<script lang="ts">
	import { hashesStore, isHashesListVisible } from '$lib/stores/hashesStore';

	// Function to format the timestamp
	const formatTimestamp = (isoString: string): string => {
		const date = new Date(isoString);
		return date.toLocaleString(); // Adjust this to your preferred format
	};

	// Function to format the hash
	const formatHash = (hash: string): string => {
		return `${hash.slice(0, 6)}...${hash.slice(-6)}`;
	};

	// Subscribe to the logs store
	let hashes: { hash: string; timestamp: string }[] = [];
	hashesStore.subscribe((value) => {
		hashes = value;
	});

	const closeHashesSidebar = () => {
		isHashesListVisible.set(false);
	};
</script>

<button
	on:click={closeHashesSidebar}
	type="button"
	class="fixed right-0 top-0 h-full w-[280px] overflow-y-auto bg-[#2d2d2d] p-5 text-surface-300 shadow-md"
>
	<div class="mb-1 font-bold">Block Hashes</div>
	{#each hashes as { hash, timestamp }}
		<div class="mb-4 rounded bg-[#343434] p-2.5 shadow">
			<div>
				<strong>Hash:</strong> <span class="font-mono text-emerald-400">{formatHash(hash)}</span>
			</div>
			<div>{formatTimestamp(timestamp)}</div>
		</div>
	{/each}
</button>
