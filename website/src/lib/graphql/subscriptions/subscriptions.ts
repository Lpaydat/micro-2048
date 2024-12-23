import { gql } from '@urql/svelte';

export const PING_SUBSCRIPTION = gql`
	subscription Notifications($chainId: ID!) {
		notifications(chainId: $chainId)
	}
`;
