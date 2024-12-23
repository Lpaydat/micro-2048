export const setBoardId = (boardId: string, leaderboardId = '') => {
	const key = leaderboardId ? `boardId-${leaderboardId}` : 'boardId';
	if (typeof window !== 'undefined') {
		localStorage.setItem(key, boardId);
	}
};

export const getBoardId = (leaderboardId = '') => {
	const key = leaderboardId ? `boardId-${leaderboardId}` : 'boardId';
	if (typeof window !== 'undefined') {
		return localStorage.getItem(key);
	}
	return null;
};
