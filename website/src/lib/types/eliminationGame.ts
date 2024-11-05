export type EliminationGameDetails = {
	name: string;
	playerCount: number;
	maxPlayers: number;
	hostName: string;
	createdAt: Date;
	totalRounds: number;
	eliminatedPerTrigger: number;
	triggerInterval: number;
};
