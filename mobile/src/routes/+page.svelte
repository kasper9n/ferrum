<script lang="ts">
	import commands from '#lib/commands.ts'
	import { open } from '@tauri-apps/plugin-dialog'
	import type { LibraryTauri } from '../../bindings'
	import { readTextFile } from '@tauri-apps/plugin-fs'
	import Library from './Library.svelte'
	import { Store } from '@tauri-apps/plugin-store'
	import { platform } from '@tauri-apps/plugin-os'

	const platform_value = platform()
	if (platform_value === 'android' || platform_value === 'ios') {
		await import('@saurl/tauri-plugin-safe-area-insets-css-api')
	}

	type StreamingService = 'spotify' | 'youtube-music'

	let loading = $state(false)
	let error = $state('')
	let streaming_service = $state<StreamingService>('spotify')

	const store = await Store.load('settings.json')
	let library = $state<LibraryTauri | null>(null)

	const saved_library_path = await store.get('library_path')
	const saved_streaming_service = await store.get('streaming_service')
	if (saved_streaming_service === 'spotify' || saved_streaming_service === 'youtube-music') {
		streaming_service = saved_streaming_service
	}
	if (typeof saved_library_path === 'string') {
		load_library(saved_library_path)
	}

	async function set_streaming_service(service: StreamingService) {
		streaming_service = service
		await store.set('streaming_service', service)
		await store.save()
	}

	async function open_library() {
		let path: string | null = null
		if (platform() === 'android') {
			const result = await commands.openFilePersistentAndroid()
			if (result.status === 'ok') {
				path = result.data
			} else {
				error = result.error
			}
		} else {
			path = await open({
				filters: [{ name: 'JSON', extensions: ['json'] }],
			})
		}
		if (path) {
			load_library(path)
		}
	}
	async function load_library(path: string) {
		loading = true
		library = null
		error = ''
		try {
			const contents = await readTextFile(path)
			const result = await commands.loadLibrary(contents)
			if (result.status === 'ok') {
				library = result.data
				store.set('library_path', path)
				store.save()
			} else {
				error = result.error
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load library. ' + String(e)
		} finally {
			loading = false
		}
	}
</script>

<div
	class="flex h-screen flex-col overflow-hidden bg-white pt-[env(safe-area-inset-top)] pb-[env(safe-area-inset-bottom)] text-sm text-neutral-800 scheme-light-dark dark:bg-neutral-950 dark:text-neutral-200"
	style:padding-top="var(--safe-area-inset-top)"
	style:padding-bottom="var(--safe-area-inset-bottom)"
>
	{#if error}
		<div class="shrink-0 border-b border-red-900 bg-red-950 px-4 py-2.5 text-xs text-red-400">
			⚠ {error}
		</div>
	{/if}

	{#snippet open_button()}
		<button
			type="button"
			onclick={open_library}
			disabled={loading}
			class="shrink-0 rounded-lg bg-neutral-900 px-3 py-1.5 text-xs font-semibold text-neutral-100 transition-colors hover:bg-neutral-700 disabled:cursor-not-allowed disabled:opacity-40 dark:bg-neutral-100 dark:text-neutral-900 dark:hover:bg-white"
		>
			{loading ? 'Loading…' : 'Open'}
		</button>
	{/snippet}

	<div class="shrink-0 border-b border-neutral-200 px-4 py-2.5 dark:border-neutral-800">
		<div class="flex items-center gap-2">
			<label for="streaming-service" class="text-xs text-neutral-500">Streaming</label>
			<select
				id="streaming-service"
				class="rounded-md border border-neutral-300 bg-white px-2 py-1 text-xs text-neutral-700 dark:border-neutral-700 dark:bg-neutral-900 dark:text-neutral-200"
				value={streaming_service}
				onchange={(event) => {
					const service = (event.currentTarget as HTMLSelectElement).value
					if (service === 'spotify' || service === 'youtube-music') {
						set_streaming_service(service)
					}
				}}
			>
				<option value="spotify">Spotify</option>
				<option value="youtube-music">YouTube Music</option>
			</select>
		</div>
	</div>

	{#if library}
		<Library {library} {open_button} {streaming_service} />
	{:else}
		<div
			class="flex flex-1 flex-col items-center justify-center gap-4 px-8 text-center text-neutral-400 dark:text-neutral-700"
		>
			<span class="text-5xl">♪</span>
			<div>
				<p class="font-medium text-neutral-500 dark:text-neutral-500">No library loaded</p>
				<div class="mt-4">
					{@render open_button()}
				</div>
			</div>
		</div>
	{/if}
</div>
