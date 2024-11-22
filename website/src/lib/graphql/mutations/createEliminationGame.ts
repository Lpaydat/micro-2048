import { Client, gql, mutationStore } from '@urql/svelte';

const CREATE_GAME = gql`
	mutation CreateEliminationGame(
		$player: String!
		$passwordHash: String!
		$settings: EliminationGameSettings!
	) {
		createEliminationGame(player: $player, passwordHash: $passwordHash, settings: $settings)
	}
`;

export type EliminationGameSettings = {
	gameName: string;
	totalRound: number;
	maxPlayers: number;
	eliminatedPerTrigger: number;
	triggerIntervalSeconds: number;
	createdTime: string;
};

export const createEliminationGame = (client: Client, settings: EliminationGameSettings) => {
	const player = localStorage.getItem('username');
	const passwordHash = localStorage.getItem('passwordHash');

	if (!player || !passwordHash) {
		console.error('Player or password hash not found');
		return;
	}

	mutationStore({ client, query: CREATE_GAME, variables: { player, passwordHash, settings } });
};
