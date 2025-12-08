import type { Client } from '@urql/svelte';
import { gql, mutationStore } from '@urql/svelte';

const CLAIM_CHAIN = gql`
	mutation ClaimChain {
		claimChain
	}
`;

export const claimChain = (client: Client) => {
	return mutationStore({
		client,
		query: CLAIM_CHAIN
	});
};
