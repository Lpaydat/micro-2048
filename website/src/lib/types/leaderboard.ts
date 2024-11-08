export type PlayerStats = {
	username: string;
	score: number;
	isEliminated: boolean;
};

export type RoundResults = {
	round: number;
	players: PlayerStats[];
	eliminatedPlayers: PlayerStats[];
};
