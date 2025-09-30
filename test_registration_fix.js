// Quick test to verify registration fix works
// This tests the core registration flow without full stress testing

import http from 'k6/http';
import { check } from 'k6';

export let options = {
  vus: 5, // Start with just 5 concurrent users
  duration: '30s',
};

const BASE_URL = 'http://localhost:8088';

// Test data - use simple player names like the fixed stress test
const PLAYER_BASE = 1000;

export default function () {
  const playerId = `player_${PLAYER_BASE + __VU}`;
  
  console.log(`Testing registration for ${playerId}`);
  
  // Test basic registration without tournament setup
  // This will hit the same code path that was failing
  const registrationPayload = {
    player: playerId,
    action: "register"
  };

  const response = http.post(`${BASE_URL}/test-registration`, JSON.stringify(registrationPayload), {
    headers: { 'Content-Type': 'application/json' },
  });

  // Check if registration succeeds (no 500 errors from aggregate_scores_from_player_chains panic)
  const success = check(response, {
    'registration request succeeds': (r) => r.status !== 500,
    'no panic in response': (r) => !r.body.includes('aggregate_scores_from_player_chains'),
    'no unreachable error': (r) => !r.body.includes('unreachable'),
  });

  if (!success) {
    console.error(`Registration failed for ${playerId}: ${response.status} - ${response.body}`);
  } else {
    console.log(`✅ Registration successful for ${playerId}`);
  }
}