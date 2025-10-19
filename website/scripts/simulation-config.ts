// ========================================
// SIMULATION CONFIGURATION PRESETS
// ========================================

export interface SimulationConfig {
	name: string;
	description: string;
	environment: 'local' | 'production';
	tournamentId?: string;
	botPersonality: 'aggressive' | 'strategic' | 'casual' | 'mixed';
	gamesPerBot: number;
	movesPerGame: number;
	loadProfile: {
		stages: Array<{
			target: number;
			duration: string;
		}>;
	};
}

export const SIMULATION_PRESETS: Record<string, SimulationConfig> = {
	// ========================================
	// LOCAL TESTING PRESETS
	// ========================================

	'local-light': {
		name: 'Local Light Load',
		description: 'Light load for local development testing',
		environment: 'local',
		botPersonality: 'mixed',
		gamesPerBot: 2,
		movesPerGame: 20,
		loadProfile: {
			stages: [
				{ target: 5, duration: '30s' },
				{ target: 10, duration: '1m' },
				{ target: 5, duration: '30s' },
				{ target: 1, duration: '10s' }
			]
		}
	},

	'local-medium': {
		name: 'Local Medium Load',
		description: 'Medium load for local stress testing',
		environment: 'local',
		botPersonality: 'mixed',
		gamesPerBot: 3,
		movesPerGame: 30,
		loadProfile: {
			stages: [
				{ target: 10, duration: '30s' },
				{ target: 25, duration: '2m' },
				{ target: 25, duration: '3m' },
				{ target: 10, duration: '1m' },
				{ target: 1, duration: '30s' }
			]
		}
	},

	// ========================================
	// PRODUCTION PRESETS
	// ========================================

	'play-along': {
		name: 'Play Along Bots',
		description: 'Realistic bot load for playing alongside humans',
		environment: 'production',
		botPersonality: 'mixed',
		gamesPerBot: 3,
		movesPerGame: 50,
		loadProfile: {
			stages: [
				{ target: 10, duration: '1m' },
				{ target: 20, duration: '5m' },
				{ target: 30, duration: '10m' },
				{ target: 20, duration: '5m' },
				{ target: 10, duration: '2m' },
				{ target: 5, duration: '1m' }
			]
		}
	},

	'aggressive-bots': {
		name: 'Aggressive Bots',
		description: 'Fast-playing aggressive bots for high activity',
		environment: 'production',
		botPersonality: 'aggressive',
		gamesPerBot: 5,
		movesPerGame: 100,
		loadProfile: {
			stages: [
				{ target: 20, duration: '30s' },
				{ target: 50, duration: '2m' },
				{ target: 50, duration: '5m' },
				{ target: 30, duration: '2m' },
				{ target: 10, duration: '1m' }
			]
		}
	},

	'strategic-bots': {
		name: 'Strategic Bots',
		description: 'Thoughtful strategic bots for realistic gameplay',
		environment: 'production',
		botPersonality: 'strategic',
		gamesPerBot: 2,
		movesPerGame: 40,
		loadProfile: {
			stages: [
				{ target: 15, duration: '1m' },
				{ target: 30, duration: '3m' },
				{ target: 30, duration: '10m' },
				{ target: 20, duration: '3m' },
				{ target: 10, duration: '2m' }
			]
		}
	},

	'stress-test': {
		name: 'Stress Test',
		description: 'High load stress testing for performance evaluation',
		environment: 'production',
		botPersonality: 'mixed',
		gamesPerBot: 5,
		movesPerGame: 80,
		loadProfile: {
			stages: [
				{ target: 20, duration: '30s' },
				{ target: 50, duration: '2m' },
				{ target: 100, duration: '5m' },
				{ target: 150, duration: '10m' },
				{ target: 100, duration: '5m' },
				{ target: 50, duration: '3m' },
				{ target: 20, duration: '2m' },
				{ target: 10, duration: '1m' }
			]
		}
	},

	'endurance-test': {
		name: 'Endurance Test',
		description: 'Long-duration test for stability evaluation',
		environment: 'production',
		botPersonality: 'mixed',
		gamesPerBot: 3,
		movesPerGame: 60,
		loadProfile: {
			stages: [
				{ target: 30, duration: '5m' },
				{ target: 50, duration: '30m' },
				{ target: 75, duration: '30m' },
				{ target: 50, duration: '15m' },
				{ target: 30, duration: '10m' },
				{ target: 10, duration: '5m' }
			]
		}
	},

	'tournament-focus': {
		name: 'Tournament Focus',
		description: 'Focused testing on specific tournament',
		environment: 'production',
		tournamentId: '', // Set this to your tournament ID
		botPersonality: 'mixed',
		gamesPerBot: 4,
		movesPerGame: 60,
		loadProfile: {
			stages: [
				{ target: 25, duration: '2m' },
				{ target: 50, duration: '8m' },
				{ target: 75, duration: '15m' },
				{ target: 50, duration: '5m' },
				{ target: 25, duration: '3m' }
			]
		}
	}
};

// ========================================
// UTILITY FUNCTIONS
// ========================================

export function getPreset(name: string): SimulationConfig | null {
	return SIMULATION_PRESETS[name] || null;
}

export function listPresets(): string[] {
	return Object.keys(SIMULATION_PRESETS);
}

export function generateK6Options(config: SimulationConfig) {
	return {
		scenarios: {
			simulation: {
				executor: 'ramping-vus',
				startVUs: 1,
				stages: config.loadProfile.stages
			}
		},
		thresholds: {
			http_req_duration: ['p(95)<3000'], // 95% of requests under 3s
			http_req_failed: ['rate<0.15'] // Error rate under 15%
		}
	};
}

export function generateEnvironmentVars(config: SimulationConfig): Record<string, string> {
	return {
		ENVIRONMENT: config.environment,
		TOURNAMENT_ID: config.tournamentId || '',
		BOT_PERSONALITY: config.botPersonality,
		GAMES_PER_BOT: config.gamesPerBot.toString(),
		MOVES_PER_GAME: config.movesPerGame.toString()
	};
}
