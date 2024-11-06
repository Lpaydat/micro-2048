<script>
	import { getClient } from '$lib/client';
	import { setContext } from 'svelte';
	import '../app.css';
	import { initializeStores, Modal } from '@skeletonlabs/skeleton';
	import { setContextClient } from '@urql/svelte';

	initializeStores();

	const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
	const applicationId = '8871dee65d6dcedf03f1e255fe7f8e69152ac1678170302283f357885b1f834407b28b27d287fbea47e91b785d2e24cead390e16fedc630e996fb03d21aa2219e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65a80100000000000000000000';
	const port = '8080';

	const client = getClient(chainId, applicationId, port);
	setContextClient(client);

	setContext('player', {
		chainId,
		username: 'lpaydat'
	})
</script>

<Modal />

<slot></slot>
