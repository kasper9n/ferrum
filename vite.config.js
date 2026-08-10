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
		minify: false, // For easier crash messages
		target: 'chrome106',
	},
	plugins: [
		sveltekit(),
		tailwindcss(),
		electron({
			entry: ['./src/electron/main.ts', './src/electron/preload.ts'],
			onstart({ startup }) {
				// @ts-expect-error Global object from vite-plugin-electorn
				const electron_app = process.electronApp
				if (electron_app) {
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
