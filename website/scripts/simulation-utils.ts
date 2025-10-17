// ========================================
// SIMULATION UTILITY FUNCTIONS
// ========================================

import { SimulationConfig, generateEnvironmentVars } from './simulation-config';

// ========================================
// BOT PERSONALITY UTILITIES
// ========================================

export interface BotPersonality {
  moveDelay: { min: number; max: number };
  batchSize: number;
  thinkingTime: number;
  namePrefixes: string[];
}

export const BOT_PERSONALITIES: Record<string, BotPersonality> = {
  aggressive: {
    moveDelay: { min: 0.5, max: 2.0 },
    batchSize: 20,
    thinkingTime: 0.1,
    namePrefixes: ['Speed', 'Flash', 'Turbo', 'Rapid', 'Quick', 'Fast', 'Lightning']
  },
  strategic: {
    moveDelay: { min: 3.0, max: 8.0 },
    batchSize: 5,
    thinkingTime: 2.0,
    namePrefixes: ['Think', 'Smart', 'Wise', 'Clever', 'Brain', 'Logic', 'Strategy']
  },
  casual: {
    moveDelay: { min: 1.0, max: 10.0 },
    batchSize: 10,
    thinkingTime: 1.0,
    namePrefixes: ['Chill', 'Relax', 'Easy', 'Cool', 'Zen', 'Calm', 'Peace']
  },
  mixed: {
    moveDelay: { min: 0.5, max: 10.0 },
    batchSize: 10,
    thinkingTime: 1.0,
    namePrefixes: ['Bot', 'Player', 'Gamer', 'User', 'AI', 'Agent', 'Entity']
  }
};

export function getRandomPersonality(): string {
  const personalities = Object.keys(BOT_PERSONALITIES);
  return personalities[Math.floor(Math.random() * personalities.length)];
}

export function getPersonalityConfig(personality: string): BotPersonality {
  return BOT_PERSONALITIES[personality] || BOT_PERSONALITIES.mixed;
}

// ========================================
// NAME GENERATION
// ========================================

export function generateBotName(personality: string): string {
  const config = getPersonalityConfig(personality);
  const prefix = config.namePrefixes[Math.floor(Math.random() * config.namePrefixes.length)];
  const suffix = Math.floor(Math.random() * 10000);
  return `${prefix}_${suffix}`;
}

export function generateRandomString(length: number): string {
  const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * characters.length));
  }
  return result;
}

// ========================================
// MOVE GENERATION
// ========================================

export function generateMoves(count: number, personality: string): string {
  const directions: string[] = ['Up', 'Right', 'Down', 'Left'];
  const config = getPersonalityConfig(personality);
  const baseTimestamp = Date.now();
  
  const moves: [string, string][] = [];
  for (let i = 0; i < count; i++) {
    let direction: string;
    
    if (personality === 'strategic') {
      // Strategic bots prefer Down and Right (generally better in 2048)
      direction = Math.random() < 0.6 
        ? (Math.random() < 0.5 ? 'Down' : 'Right')
        : directions[Math.floor(Math.random() * directions.length)];
    } else if (personality === 'aggressive') {
      // Aggressive bots use more random moves for faster gameplay
      direction = directions[Math.floor(Math.random() * directions.length)];
    } else {
      // Mixed/casual behavior
      direction = directions[Math.floor(Math.random() * directions.length)];
    }
    
    const timestamp = baseTimestamp + (i * 1000) + (Math.random() * 500);
    moves.push([direction, timestamp.toString()]);
  }
  
  return JSON.stringify(moves);
}

// ========================================
// API CONFIGURATION
// ========================================

export interface ApiConfig {
  website: string;
  port: string;
  chainId: string;
  applicationId: string;
}

export function getApiConfig(environment: 'local' | 'production'): ApiConfig {
  const configs = {
    local: {
      website: 'localhost',
      port: '8080',
      chainId: '7b9613d4da9ea6adb4399cb61a4fcc831775ce80fa11372219d323463c4ef130',
      applicationId: '4e6d771d3d1a21d04038df2250b1fea5e4f061d2108ee552a2ea1c41fdc86aad600f140ec9832578b8343490a00c6573e02f17407429d3bffcad7ae92badf70dc7ca03c01755334323956b5413e24d9a0404b9292fcc91e60700abe80bd5f7a8030000000000000000000000'
    },
    production: {
      website: 'api.micro2048.xyz',
      port: '443',
      chainId: '7b9613d4da9ea6adb4399cb61a4fcc831775ce80fa11372219d323463c4ef130',
      applicationId: '409c67886ae3881c8d03b41c12f27b6cfe3b28c5ea89385aa9eb126155c9a9c9'
    }
  };
  
  return configs[environment];
}

export function buildApiUrl(config: ApiConfig): string {
  return `https://${config.website}:${config.port}/chains/${config.chainId}/applications/${config.applicationId}`;
}

export function buildPlayerApiUrl(config: ApiConfig, playerChainId: string): string {
  return `https://${config.website}:${config.port}/chains/${playerChainId}/applications/${config.applicationId}`;
}

// ========================================
// TOURNAMENT UTILITIES
// ========================================

export interface Tournament {
  tournamentId: string;
  name: string;
  startTime: number;
  endTime: number;
  isActive: boolean;
}

export function selectBestTournament(tournaments: Tournament[]): Tournament | null {
  if (!tournaments || tournaments.length === 0) {
    return null;
  }
  
  // Filter active tournaments
  const activeTournaments = tournaments.filter(t => t.isActive);
  
  if (activeTournaments.length === 0) {
    return null;
  }
  
  // For now, return the first active tournament
  // In a more sophisticated implementation, you might:
  // - Select based on player count
  // - Select based on time remaining
  // - Select based on tournament type
  return activeTournaments[0];
}

// ========================================
// COMMAND LINE HELPERS
// ========================================

export function buildK6Command(config: SimulationConfig, presetName: string): string {
  const envVars = generateEnvironmentVars(config);
  const envString = Object.entries(envVars)
    .map(([key, value]) => `${key}="${value}"`)
    .join(' ');
  
  return `${envString} k6 run website/scripts/simulation.ts`;
}

export function printUsage() {
  console.log(`
🤖 2048 Simulation Test Usage:

📋 Available Presets:
${Object.keys(require('./simulation-config').SIMULATION_PRESETS).map(name => `  • ${name}`).join('\n')}

🎮 Quick Start Commands:

# Play alongside bots (recommended for human interaction)
ENVIRONMENT=production BOT_PERSONALITY=mixed GAMES_PER_BOT=3 k6 run --vus 20 --duration 30m website/scripts/simulation.ts

# Local testing
ENVIRONMENT=local BOT_PERSONALITY=mixed GAMES_PER_BOT=2 k6 run --vus 10 --duration 5m website/scripts/simulation.ts

# Aggressive bots for high activity
ENVIRONMENT=production BOT_PERSONALITY=aggressive GAMES_PER_BOT=5 k6 run --vus 30 --duration 15m website/scripts/simulation.ts

# Strategic bots for realistic gameplay
ENVIRONMENT=production BOT_PERSONALITY=strategic GAMES_PER_BOT=2 k6 run --vus 25 --duration 20m website/scripts/simulation.ts

# Stress testing
ENVIRONMENT=production BOT_PERSONALITY=mixed GAMES_PER_BOT=5 k6 run --vus 100 --duration 1h website/scripts/simulation.ts

🎯 Environment Variables:
  ENVIRONMENT           'local' or 'production' (default: production)
  TOURNAMENT_ID         Specific tournament ID (default: auto-discover)
  BOT_PERSONALITY       'aggressive', 'strategic', 'casual', 'mixed' (default: mixed)
  GAMES_PER_BOT         Number of games per bot (default: 3)
  MOVES_PER_GAME        Number of moves per game (default: 50)

💡 Tips:
• Start with 'play-along' scenario for human interaction
• Use 'local' environment for development testing
• Monitor bot names in logs to see different personalities
• Adjust VUs and duration based on your needs
`);
}

// ========================================
// LOGGING UTILITIES
// ========================================

export function logBotStart(username: string, personality: string) {
  console.log(`🤖 Bot ${username} (${personality}) starting simulation...`);
}

export function logBotSuccess(username: string) {
  console.log(`✅ ${username} completed simulation`);
}

export function logBotError(username: string, error: string) {
  console.error(`❌ ${username} failed: ${error}`);
}

export function logTournamentFound(tournament: Tournament) {
  console.log(`🎯 Selected tournament: ${tournament.name} (${tournament.tournamentId.substring(0, 16)}...)`);
}

export function logGameStart(username: string, gameCount: number) {
  console.log(`🎮 ${username} creating ${gameCount} games...`);
}

export function logBoardPlay(username: string, boardId: string) {
  console.log(`🎲 ${username} playing board ${boardId.substring(0, 8)}...`);
}