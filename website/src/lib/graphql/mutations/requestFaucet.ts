import type { Client } from '@urql/svelte';
import { gql, mutationStore } from '@urql/svelte';

const REQUEST_FAUCET = gql`
	mutation RequestFaucet {
		faucet
	}
`;

export const requestFaucetMutation = (client: Client) => {
	return mutationStore({
		client,
		query: REQUEST_FAUCET
	});
};
