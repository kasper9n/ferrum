import adapter from '@sveltejs/adapter-static'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

export default {
	adapter: adapter(),
	// Consult https://svelte.dev/docs#compile-time-svelte-preprocess
	// for more information about preprocessors
	preprocess: vitePreprocess(),
	kit: {
		alias: {
			$routes: 'src/routes',
			$components: 'src/components',
			$electron: 'src/electron',
		},
	},
	compilerOptions: {
		experimental: {
			async: true,
		},
	},
}
