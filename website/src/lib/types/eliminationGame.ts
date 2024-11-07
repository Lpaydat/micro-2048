export type EliminationGameDetails = {
	gameName: string;
	playerCount: number;
	maxPlayers: number;
	host: string;
	createdAt: Date;
	currentRound?: number;
	totalRounds: number;
	eliminatedPerTrigger: number;
	triggerIntervalSeconds: number;
};
