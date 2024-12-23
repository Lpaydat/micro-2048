import { writable } from 'svelte/store';

export const userStore = writable<{
	username: string | undefined | null;
	passwordHash: string | undefined | null;
	chainId: string | undefined | null;
	isMod: boolean | undefined | null;
}>({
	username: undefined,
	passwordHash: undefined,
	chainId: undefined,
	isMod: undefined
});
