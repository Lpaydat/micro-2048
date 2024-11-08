import { Client } from 'urql';

import { gql, queryStore } from '@urql/svelte';

const GET_GAME = gql`
	query GetEliminationGameDetails($gameId: String!, $round: Int!) {
		eliminationGame(gameId: $gameId, round: $round) {
			gameId
			chainId
			gameName
			host
			players
			maxPlayers
			currentRound
			totalRounds
			triggerIntervalSeconds
			eliminatedPerTrigger
			createdTime
			status
			gameLeaderboard {
				username
				score
			}
			roundLeaderboard {
				round
				players {
					username
					score
				}
				eliminatedPlayers {
					username
					score
				}
			}
		}
	}
`;

export const getGameDetails = (client: Client, gameId: string, round: number = 0) => {
	return queryStore({ client, query: GET_GAME, variables: { gameId, round } });
};
