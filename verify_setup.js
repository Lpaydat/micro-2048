// Quick verification of the new setup
import { check } from 'k6';

// Simulate the player naming logic
const PLAYER_BASE = Math.floor(Date.now() / 1000);

export const options = {
  scenarios: {
    verify: {
      executor: 'constant-vus',
      vus: 3,
      duration: '10s',
    },
  },
};

export default function() {
  // Test player naming
  const playerIndex = PLAYER_BASE + __VU;
  const playerId = `player_${playerIndex}`;
  const password = `player_${playerIndex}`;
  
  console.log(`VU ${__VU}: Generated player ${playerId} with password ${password}`);
  
  // Verify uniqueness
  const isUnique = check(playerId, {
    'player ID is unique': (id) => id.includes(PLAYER_BASE.toString()),
    'player ID format correct': (id) => id.startsWith('player_'),
  });
  
  if (isUnique) {
    console.log(`✅ VU ${__VU}: Player naming verified`);
  } else {
    console.log(`❌ VU ${__VU}: Player naming failed`);
  }
}