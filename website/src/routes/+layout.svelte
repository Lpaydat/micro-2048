<script>
	import { getClient } from '$lib/client';
	import { setContext } from 'svelte';
	import '../app.css';
	import { initializeStores, Modal } from '@skeletonlabs/skeleton';
	import { setContextClient } from '@urql/svelte';

	initializeStores();

	const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
	const applicationId = '2d2aabc26288a3e34d0fc0397f3987c18226e2ea049b46eef36039c719279f7d8db587fd8b0c2a15b407119dd234c3fcf5502f32e78e97fd3fa8574d370768c2e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';
	const port = '8080';

	const client = getClient(chainId, applicationId, port);
	setContextClient(client);

	setContext('player', {
		chainId,
		username: 'lpaydat'
	});
</script>

<Modal />

<slot />
