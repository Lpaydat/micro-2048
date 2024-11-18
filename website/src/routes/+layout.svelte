<script lang="ts">
	import { getClient } from '$lib/client';
	import { onMount } from 'svelte';
	import '../app.css';
	import { initializeStores, Modal } from '@skeletonlabs/skeleton';
	import { setContextClient } from '@urql/svelte';
	import { userStore } from '$lib/stores/userStore';

	initializeStores();

	// const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
	// const applicationId = '6c6bd0bf320bf7cc0d2f972f1649b9e52a151f2dbfc50bbdf736242405dcb268341717e586aaace9516bd152eb494c5ac0a38db37681d643e01e90a92950513de476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';
	// const port = '8080';
	const chainId = 'e476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65';
	const applicationId = '6c6bd0bf320bf7cc0d2f972f1649b9e52a151f2dbfc50bbdf736242405dcb268341717e586aaace9516bd152eb494c5ac0a38db37681d643e01e90a92950513de476187f6ddfeb9d588c7b45d3df334d5501d6499b3f9ad5595cae86cce16a65010000000000000000000000';
	const port = '8080';

	const client = getClient(chainId, applicationId, port);
	setContextClient(client);

	onMount(() => {
		const username = localStorage.getItem('username');
		const passwordHash = localStorage.getItem('passwordHash');
		const chainId = localStorage.getItem('chainId');

		if (username && passwordHash && chainId) {
		  userStore.update(store => ({
		    ...store,
		    username: localStorage.getItem('username'),
		    passwordHash: localStorage.getItem('passwordHash'),
		    chainId: localStorage.getItem('chainId')
		  }))
		}
	})
</script>

<Modal />

<slot />
