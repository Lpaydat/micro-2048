import type { Client } from '@urql/svelte';
import { gql, queryStore } from '@urql/svelte';

export const GET_GAME_DETAILS = gql`
	query GetEliminationGameDetails($round: Int!) {
		eliminationGame(round: $round) {
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
			lastUpdatedTime
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

export const getGameDetails = (client: Client, round: number = 0) => {
	return queryStore({ client, query: GET_GAME_DETAILS, variables: { round } });
};
