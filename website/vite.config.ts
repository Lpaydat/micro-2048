import { defineConfig, loadEnv } from 'vite';

import { enhancedImages } from '@sveltejs/enhanced-img';
import { sveltekit } from '@sveltejs/kit/vite';

	export default defineConfig(({ mode }) => {
		// Load env file based on `mode` in the current working directory.
		const env = loadEnv(mode, process.cwd(), '');
		
		return {
			plugins: [enhancedImages(), sveltekit()],
			build: {
				sourcemap: false
			},
			server: {
				fs: {
					allow: ['..']
				}
			},
			define: {
				__APP_ENV__: JSON.stringify(env)
			},
			optimizeDeps: {
				exclude: ['@urql/core', 'wonka', 'clsx']
			}
		};
	});
