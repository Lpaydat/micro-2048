import type { Client } from '@urql/svelte';
import { gql, queryStore } from '@urql/svelte';

const GET_CHAIN_POOL_STATUS = gql`
	query GetChainPoolStatus {
		chainPoolStatus {
			poolSize
			targetSize
			lowThreshold
			needsReplenish
		}
	}
`;

export type ChainPoolStatus = {
	poolSize: number;
	targetSize: number;
	lowThreshold: number;
	needsReplenish: boolean;
};

export const getChainPoolStatus = (client: Client) => {
	return queryStore<{ chainPoolStatus: ChainPoolStatus }>({
		client,
		query: GET_CHAIN_POOL_STATUS,
		requestPolicy: 'network-only'
	});
};
