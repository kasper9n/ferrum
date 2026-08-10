import path from 'path'
import { defineConfig } from 'vite'
import { svelte, vitePreprocess } from '@sveltejs/vite-plugin-svelte'
import electron from 'vite-plugin-electron'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
	base: '/',
	clearScreen: false,
	resolve: {
		alias: {
			// These must also be specified in tsconfig.json
			$lib: path.resolve(import.meta.dirname, './src/lib'),
			$components: path.resolve(import.meta.dirname, './src/components'),
			$electron: path.resolve(import.meta.dirname, './src/electron'),
		},
	},
	server: {
		watch: {
			ignored: ['**/src-native/**'],
		},
	},
	build: {
		outDir: './build/web',
		sourcemap: true,
		minify: false, // For easier crash messages
		target: 'chrome106',
	},
	plugins: [
		svelte({
			preprocess: vitePreprocess(),
		}),
		tailwindcss(),
		electron({
			entry: ['./src/electron/main.ts', './src/electron/preload.ts'],
			onstart({ startup }) {
				if (process.electronApp) {
					process.kill(process.electronApp.pid, 'SIGTERM')
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
