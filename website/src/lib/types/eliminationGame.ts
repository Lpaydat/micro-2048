export type EliminationGameDetails = {
	gameName: string;
	playerCount: number;
	maxPlayers: number;
	host: string;
	createdTime: string;
	currentRound?: number;
	totalRounds: number;
	eliminatedPerTrigger: number;
	triggerIntervalSeconds: number;
};
