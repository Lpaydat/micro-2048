import { createClient as createWSClient } from "graphql-ws";

import { cacheExchange, Client, fetchExchange, subscriptionExchange } from "@urql/svelte";

export const getClient = (chainId: string, applicationId: string, port: string) => {
  const ipAddress = import.meta.env.VITE_IP_ADDRESS;
  const wsClient = createWSClient({
    url: `ws://${ipAddress}:${port}/ws`,
  });

  return new Client({
    url: `http://${ipAddress}:${port}/chains/${chainId}/applications/${applicationId}`,
    exchanges: [
      cacheExchange,
      fetchExchange,
      subscriptionExchange({
        forwardSubscription(request: any) {
          const input = { ...request, query: request.query || "" };
          return {
            subscribe(sink: any) {
              const unsubscribe = wsClient.subscribe(input, sink);
              return { unsubscribe };
            },
          };
        },
      }),
    ],
  });
};
