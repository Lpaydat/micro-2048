import { browser } from '$app/environment';
import { createClient as createWSClient } from 'graphql-ws';

import { cacheExchange, Client, fetchExchange, subscriptionExchange } from '@urql/svelte';

import {
	website,
	chainId as mainChainId,
	applicationId as mainApplicationId,
	port as mainPort
} from './constants';
import { userStore } from './stores/userStore';

const getBaseUrl = (website: string, port: string) => {
	const protocol = website === 'localhost' ? 'ws' : 'wss';
	const httpProtocol = website === 'localhost' ? 'http' : 'https';

	// Omit standard ports (80 for http, 443 for https)
	const shouldIncludePort =
		port &&
		!((httpProtocol === 'https' && port === '443') || (httpProtocol === 'http' && port === '80'));

	const portSuffix = shouldIncludePort ? `:${port}` : '';

	return {
		ws: `${protocol}://${website}${portSuffix}/ws`,
		http: `${httpProtocol}://${website}${portSuffix}`
	};
};

export const getClient = (
	chainId: string | undefined | null,
	useMainChainAsDefault: boolean = false,
	applicationId = mainApplicationId,
	port = mainPort
) => {
	const urls = getBaseUrl(website, port);
	let userChainId: string | null | undefined;
	const unsubscribe = userStore.subscribe((value) => {
		userChainId = value.chainId;
	});
	unsubscribe();
	chainId = chainId || (useMainChainAsDefault ? mainChainId : userChainId);

	if (!chainId) {
		throw new Error(`Chain ID is required. Got chainId: ${chainId}`);
	}

	const clientUrl = `${urls.http}/chains/${chainId}/applications/${applicationId}`;
	
	console.log('🔗 GraphQL Client URL:', clientUrl);
	console.log('🌐 Base URLs:', urls);
	console.log('⚙️ Chain ID:', chainId);
	console.log('📱 Application ID:', applicationId);

	// Create basic HTTP client for server-side rendering
	if (!browser) {
		return new Client({
			url: clientUrl,
			exchanges: [cacheExchange, fetchExchange]
		});
	}

	// Create full client with WebSocket support for browser
	const wsClient = createWSClient({
		url: urls.ws
	});

	return new Client({
		url: clientUrl,
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
