import http from 'k6/http';

const chainId = '609ea0de061d075018c140758f2e0bd956cca374213b70a7f84fa697e9711b63';
const applicationId =
	'425e11124244eada438d9afde9325ac52efb42bd3e4ffcb86a0480366406ef8192366fc6f617e8fca2a7d9298b8924df917530b957e2f295e78dfc7bcf15ee17e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';
const website = 'localhost:8080';

const API_URL = `http://${website}/chains/${chainId}/applications/${applicationId}`;

// Update options with proper thresholds and system resource management
export const options = {
	scenarios: {
		load_test: {
			executor: 'ramping-vus',
			startVUs: 300 * 16,
			stages: [
				{ target: 300 * 16, duration: '1s' } // Initial gentle warmup
			]
		}
	},
	// Add batch support and proper timeouts
	batch: 20, // Group similar requests
	batchPerHost: 10 // Prevent host overload
};

export default async function () {
	const newShard = `mutation newShard {
		newShard
	}`;

	const params = {
		timeout: '120s',
		headers: {
			'Content-Type': 'application/json',
			Connection: 'keep-alive'
		}
	};

	http.post(API_URL, JSON.stringify({ query: newShard }), params);
}
