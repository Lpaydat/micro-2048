<script>
	import { getClient } from '$lib/client';
	import { setContext } from 'svelte';
	import '../app.css';
	import { initializeStores, Modal } from '@skeletonlabs/skeleton';
	import { setContextClient } from '@urql/svelte';

	initializeStores();

	const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
	const applicationId = 'b75ac543c34884fdb003813dc1026cff269e89a43fc04624b755fe5772254dc92b1cd3b485592ab64d3132ef6c5b7acead6a1e6b4e0be50e2f4c9f8c46a77282e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65b50100000000000000000000';
	const port = '8080';

	const client = getClient(chainId, applicationId, port);
	setContextClient(client);

	setContext('player', {
		chainId,
		username: 'moii'
	})
</script>

<Modal />

<slot></slot>
