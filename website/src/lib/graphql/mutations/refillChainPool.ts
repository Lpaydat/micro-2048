import type { Client } from '@urql/svelte';
import { gql, mutationStore } from '@urql/svelte';

const REFILL_CHAIN_POOL = gql`
	mutation RefillChainPool($count: Int!) {
		refillChainPool(count: $count)
	}
`;

export const refillChainPool = (client: Client, count: number) => {
	return mutationStore({
		client,
		query: REFILL_CHAIN_POOL,
		variables: {
			count
		}
	});
};
