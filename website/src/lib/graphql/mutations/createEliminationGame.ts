import { Client, gql, mutationStore } from '@urql/svelte';

const CREATE_GAME = gql`
	mutation CreateEliminationGame($player: String!, $settings: EliminationGameSettings!) {
		createEliminationGame(player: $player, settings: $settings)
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

export const createEliminationGame = (
	client: Client,
	player: string,
	settings: EliminationGameSettings
) => {
	mutationStore({ client, query: CREATE_GAME, variables: { player, settings } });
};
