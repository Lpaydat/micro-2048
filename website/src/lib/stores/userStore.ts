import { writable } from 'svelte/store';

export type Player = {
	username: string | undefined | null;
	passwordHash: string | undefined | null;
	chainId: string | undefined | null;
	isMod: boolean | undefined | null;
};

export const userStore = writable<Player>({
	username: undefined,
	passwordHash: undefined,
	chainId: undefined,
	isMod: undefined
});

export const userBalanceStore = writable<string | null>(null);
