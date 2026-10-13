import adapter from '@sveltejs/adapter-static'
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'
import { defineConfig } from 'vite'
import { sveltekit } from '@sveltejs/kit/vite'
import electron from 'vite-plugin-electron'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
	clearScreen: false,
	server: {
		watch: {
			ignored: ['**/src-native/**'],
		},
	},
	build: {
		sourcemap: true,
		minify: false /* For easier crash messages */,
		target: 'chrome106',
	},
	plugins: [
		sveltekit({
			// Consult https://svelte.dev/docs#compile-time-svelte-preprocess
			// for more information about preprocessors
			preprocess: vitePreprocess(),

			adapter: adapter({
				pages: './build/web',
				fallback: 'index.html',
				assets: './build/web',
			}),

			alias: {
				$routes: 'src/routes',
				$components: 'src/components',
				$electron: 'src/electron',
			},
		}),
		tailwindcss(),
		electron({
			entry: ['./src/electron/main.ts', './src/electron/preload.ts'],
			onstart({ startup }) {
				const electron_app = process.electronApp
				if (electron_app?.pid) {
					process.kill(electron_app.pid, 'SIGTERM')
				} else {
					startup()
				}
			},
			vite: {
				build: {
					outDir: './build/electron',
					emptyOutDir: true,
				},
			},
		}),
	],
})
