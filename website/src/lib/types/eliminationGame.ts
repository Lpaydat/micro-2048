export type EliminationGameDetails = {
	name: string;
	playerCount: number;
	maxPlayers: number;
	hostName: string;
	createdAt: Date;
	currentRound?: number;
	totalRounds: number;
	eliminatedPerTrigger: number;
	triggerInterval: number;
};
