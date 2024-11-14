import { writable } from 'svelte/store';

export const hashesStore = writable<{ hash: string; timestamp: string }[]>([]);

export const isHashesListVisible = writable<boolean>(true);
