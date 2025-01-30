import { writable, get } from 'svelte/store';

type ShardStore = Record<string, string[]>;
const initialState: ShardStore = {};

const shardStore = writable<ShardStore>(initialState);

export const addShards = (leaderboardId: string, shards: string[]) => {
	shardStore.update((current) => ({
		...current,
		[leaderboardId]: shards
	}));
};

export const getShards = (leaderboardId: string): string[] | undefined => {
	let shards: string[] | undefined;
	shardStore.subscribe(($store) => {
		shards = $store[leaderboardId];
	})();
	return shards;
};

export const getRandomShard = async (
	leaderboardId: string,
	username: string
): Promise<string | undefined> => {
	let selectedShard: string | undefined;
	const $store = get(shardStore);
	const shards = $store[leaderboardId];

	if (shards?.length) {
		const hashBuffer = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(username));
		const hashArray = new Uint8Array(hashBuffer);
		const hashInt = new DataView(hashArray.buffer).getUint32(0);
		selectedShard = shards[Math.abs(hashInt) % shards.length];
	}

	return selectedShard;
};

export default shardStore;
