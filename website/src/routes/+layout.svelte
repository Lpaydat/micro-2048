<script>
	import { getClient } from '$lib/client';
	import { setContext } from 'svelte';
	import '../app.css';
	import { initializeStores, Modal } from '@skeletonlabs/skeleton';
	import { setContextClient } from '@urql/svelte';

	initializeStores();

	const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
	const applicationId = '2d2aabc26288a3e34d0fc0397f3987c18226e2ea049b46eef36039c719279f7dfab91bcfbb308a1d35f377c5b12caaf18c27ffeb9f8bf475ec99d3052f09deaee476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';
	const port = '8080';

	const client = getClient(chainId, applicationId, port);
	setContextClient(client);

	setContext('player', {
		chainId,
		username: 'mint'
	})
</script>

<Modal />

<slot />
