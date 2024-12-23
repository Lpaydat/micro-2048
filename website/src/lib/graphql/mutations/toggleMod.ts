import type { Client } from '@urql/svelte';
import { gql, mutationStore } from '@urql/svelte';

const TOGGLE_MOD = gql`
	mutation ToggleMod($username: String!, $player: String!, $passwordHash: String!) {
		toggleMod(username: $username, player: $player, passwordHash: $passwordHash)
	}
`;

export const toggleMod = (client: Client, username: string) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		return;
	}

	mutationStore({
		client,
		query: TOGGLE_MOD,
		variables: {
			username,
			player,
			passwordHash
		}
	});
};
