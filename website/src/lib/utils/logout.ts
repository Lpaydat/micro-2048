import { userStore } from '$lib/stores/userStore';

export const logout = () => {
	userStore.update((store) => ({
		...store,
		chainId: undefined,
		username: undefined,
		passwordHash: undefined,
		isMod: undefined
	}));

	localStorage.removeItem('username');
	localStorage.removeItem('passwordHash');
	localStorage.removeItem('chainId');
	localStorage.removeItem('boardId');
	localStorage.removeItem('isMod');
};
