export type EliminationGameDetails = {
	gameId: string;
	gameName: string;
	playerCount: number;
	maxPlayers: number;
	host: string;
	createdTime: string;
	currentRound?: number;
	totalRounds: number;
	eliminatedPerTrigger: number;
	triggerIntervalSeconds: number;
	status: 'Waiting' | 'Active' | 'Ended';
};
