<script lang="ts">
  import { hashesStore } from '$lib/stores/hashesStore';

  // Function to format the timestamp
  function formatTimestamp(isoString: string): string {
    const date = new Date(isoString);
    return date.toLocaleString(); // Adjust this to your preferred format
  }

  // Function to format the hash
  function formatHash(hash: string): string {
    return `${hash.slice(0, 6)}...${hash.slice(-6)}`;
  }

  // Subscribe to the logs store
  let hashes: { hash: string; timestamp: string }[] = [];
  hashesStore.subscribe(value => {
    hashes = value;
  });
</script>

<div class="fixed top-0 right-0 h-full w-[280px] overflow-y-auto text-surface-300 bg-[#2d2d2d] p-5 shadow-md">
  <div class="font-bold mb-1">Block Hashes</div>
  {#each hashes as { hash, timestamp }}
    <div class="mb-4 rounded bg-[#343434] p-2.5 shadow">
      <div><strong>Hash:</strong> <span class="text-emerald-400">{formatHash(hash)}</span></div>
      <div>{formatTimestamp(timestamp)}</div>
    </div>
  {/each}
</div>