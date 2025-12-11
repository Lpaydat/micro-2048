// Get username from localStorage to include in key (makes boardId per-user)
const getUsername = () => {
	if (typeof window !== 'undefined') {
		return localStorage.getItem('username') || '';
	}
	return '';
};

const getKey = (leaderboardId: string) => {
	const username = getUsername();
	if (leaderboardId && username) {
		return `boardId-${username}-${leaderboardId}`;
	} else if (leaderboardId) {
		return `boardId-${leaderboardId}`;
	} else if (username) {
		return `boardId-${username}`;
	}
	return 'boardId';
};

export const setBoardId = (boardId: string, leaderboardId = '') => {
	const key = getKey(leaderboardId);
	if (typeof window !== 'undefined') {
		localStorage.setItem(key, boardId);
	}
};

export const getBoardId = (leaderboardId = '') => {
	const key = getKey(leaderboardId);
	if (typeof window !== 'undefined') {
		return localStorage.getItem(key);
	}
	return null;
};

export const deleteBoardId = (leaderboardId = '') => {
	const key = getKey(leaderboardId);
	if (typeof window !== 'undefined') {
		localStorage.removeItem(key);
	}
};
