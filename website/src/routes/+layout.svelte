<script>
	import { getClient } from '$lib/client';
	import '../app.css';
	import { initializeStores, Modal } from '@skeletonlabs/skeleton';
	import { setContextClient } from '@urql/svelte';

	initializeStores();

	const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
	const applicationId = '0d092db7604d493c214124823867b03d4d18d3b3bfd48aee8b420a4636f4feffcd3f7a870a80d801480c26d215e514c503bfd8bf586718408d62bc106c8d1d56e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65dd0600000000000000000000';
	const port = '8080';

	const client = getClient(chainId, applicationId, port);
	setContextClient(client);
</script>

<Modal />

<slot />
