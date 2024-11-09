import { Client, gql, queryStore } from '@urql/svelte';

const GET_PLAYER = gql`
	query GetPlayer($username: String!) {
		player(username: $username) {
			username
			chainId
		}
	}
`;

export const getPlayerInfo = (client: Client, username: string) => {
	return queryStore({ client, query: GET_PLAYER, variables: { username } });
};
