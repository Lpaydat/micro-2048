import { browser } from '$app/environment';
import { createClient as createWSClient } from 'graphql-ws';

import { cacheExchange, Client, fetchExchange, subscriptionExchange } from '@urql/svelte';

const getBaseUrl = (ipAddress: string, port: string) => ({
	ws: `ws://${ipAddress}:${port}/ws`,
	http: `http://${ipAddress}:${port}`
});

export const getClient = (chainId: string, applicationId: string, port: string) => {
	const ipAddress = '64.225.79.45'; // or import.meta.env.VITE_IP_ADDRESS
	const urls = getBaseUrl(ipAddress, port);

	// Create basic HTTP client for server-side rendering
	if (!browser) {
		return new Client({
			url: `${urls.http}/chains/${chainId}/applications/${applicationId}`,
			exchanges: [cacheExchange, fetchExchange]
		});
	}

	// Create full client with WebSocket support for browser
	const wsClient = createWSClient({
		url: urls.ws
	});

	return new Client({
		url: `${urls.http}/chains/${chainId}/applications/${applicationId}`,
		exchanges: [
			cacheExchange,
			fetchExchange,
			subscriptionExchange({
				forwardSubscription(request) {
					const input = { ...request, query: request.query || '' };
					return {
						subscribe(sink) {
							const unsubscribe = wsClient.subscribe(input, sink);
							return { unsubscribe };
						}
					};
				}
			})
		]
	});
};
