export const setBoardId = (boardId: string, leaderboardId = '') => {
	const key = leaderboardId ? `boardId-${leaderboardId}` : 'boardId';
	localStorage.setItem(key, boardId);
};

export const getBoardId = (leaderboardId = '') => {
	const key = leaderboardId ? `boardId-${leaderboardId}` : 'boardId';
	return localStorage.getItem(key);
};
