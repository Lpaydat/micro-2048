import { Client, gql, mutationStore } from '@urql/svelte';

const NEW_SHARD = gql`
	mutation NewShard {
		newShard
	}
`;

export const newShard = (client: Client) => {
	mutationStore({
		client,
		query: NEW_SHARD
	});
};
