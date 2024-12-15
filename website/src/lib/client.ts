import { browser } from '$app/environment';
import { createClient as createWSClient } from 'graphql-ws';

import { cacheExchange, Client, fetchExchange, subscriptionExchange } from '@urql/svelte';

import { website, chainId as mainChainId } from './constants';
import { userStore } from './stores/userStore';

const getBaseUrl = (website: string, port: string) => {
	const protocol = website === 'localhost' ? 'ws' : 'wss';
	const httpProtocol = website === 'localhost' ? 'http' : 'https';

	return {
		ws: `${protocol}://${website}:${port}/ws`,
		http: `${httpProtocol}://${website}:${port}`
	};
};

export const getClient = (
	chainId: string | undefined | null,
	applicationId: string,
	port: string,
	allowDefaultChainId: boolean = false
) => {
	const urls = getBaseUrl(website, port);
	let userChainId;
	userStore.subscribe((value) => {
		userChainId = value.chainId;
	})();
	chainId = chainId || (allowDefaultChainId ? mainChainId : userChainId);

	if (!chainId) {
		throw new Error(`Chain ID is required. Got chainId: ${chainId}`);
	}

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
		fetchOptions: {
			headers: {
				'Content-Type': 'application/json'
			}
		},
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
