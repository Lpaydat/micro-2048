import type { Config } from 'tailwindcss';

import { skeleton } from "@skeletonlabs/tw-plugin";

const config = {
	darkMode: 'class',
	content: [
		'./src/**/*.{html,js,svelte,ts}',
		'./node_modules/@skeletonlabs/skeleton/**/*.{html,js,svelte,ts}'
	],
	theme: {
		extend: {}
	},
	plugins: [
		skeleton({
			themes: { preset: ['crimson'] }
		})
	]
} satisfies Config;

export default config;
