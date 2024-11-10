import { writable } from 'svelte/store';

export const userStore = writable<{
	username: string | undefined;
	passwordHash: string | undefined;
	chainId: string | undefined;
}>({
	username: undefined,
	passwordHash: undefined,
	chainId: undefined
});
