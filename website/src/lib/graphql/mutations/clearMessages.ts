import { getClient } from '$lib/client';

import { gql, mutationStore } from '@urql/svelte';

const CLEAR_MESSAGES = gql`
	mutation ClearMessages {
		clearMessages
	}
`;

export const clearMessages = (chainId: string, applicationId: string, port: string) => {
	const playerClient = getClient(chainId, applicationId, port);
	mutationStore({ client: playerClient, query: CLEAR_MESSAGES });
};
