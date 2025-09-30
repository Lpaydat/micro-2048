// Simple K6 test for single player registration to debug the issue
import http from "k6/http";
import { check, sleep } from "k6";

export const options = {
  scenarios: {
    single_registration: {
      executor: "constant-vus",
      vus: 1,
      duration: "30s",
      maxDuration: "1m",
    },
  },
};

export default function () {
  const testConfig = JSON.parse(open("./stress_test_config.json"));
  const apiConfig = testConfig.api;
  const mainUrl = `${apiConfig.base_url}/chains/${apiConfig.chain_id}/applications/${apiConfig.app_id}`;
  
  // Use the same unique naming pattern as the phased test
  const TEST_SESSION_ID = Math.random().toString(36).substring(2, 8);
  const playerId = `test_player_${TEST_SESSION_ID}_${__VU}_${__ITER}`;
  const password = `test_password_${TEST_SESSION_ID}`;
  
  const registerQuery = `
    mutation RegisterPlayer($username: String!, $passwordHash: String!) {
      registerPlayer(username: $username, passwordHash: $passwordHash)
    }
  `;
  
  console.log(`🔐 Testing registration for: ${playerId}`);
  
  const payload = {
    query: registerQuery,
    variables: { username: playerId, passwordHash: password }
  };
  
  const params = {
    headers: {
      "Content-Type": "application/json",
    },
    timeout: "30s",
  };
  
  const response = http.post(mainUrl, JSON.stringify(payload), params);
  
  console.log(`📊 Response Status: ${response.status}`);
  if (response.body) {
    console.log(`📄 Response Body: ${response.body.substring(0, 500)}`);
  }
  
  const success = check(response, {
    "registration successful": (r) => r.status === 200,
    "no errors in response": (r) => {
      if (r.body) {
        try {
          const parsed = JSON.parse(r.body);
          return !parsed.errors || parsed.errors.length === 0;
        } catch (e) {
          return false;
        }
      }
      return false;
    },
  });
  
  if (success) {
    console.log(`✅ Registration successful for ${playerId}`);
  } else {
    console.log(`❌ Registration failed for ${playerId}`);
  }
  
  sleep(2);
}