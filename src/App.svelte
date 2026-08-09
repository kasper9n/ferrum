<script lang="ts">
	import { onDestroy, onMount } from 'svelte'
	import { fade, fly } from 'svelte/transition'
	import TrackList from './components/TrackList.svelte'
	import Player from './components/Player.svelte'
	import Sidebar from './components/Sidebar.svelte'
	import Queue from './components/Queue.svelte'
	import Lyrics from './components/Lyrics.svelte'
	import TrackInfo, { track_info_state } from './components/TrackInfo.svelte'
	import PlaylistInfoModal from './components/PlaylistInfo.svelte'
	import { queue_visible } from './lib/queue'
	import { lyrics_state } from '$lib/lyrics.svelte'
	import { ipc_listen, ipc_renderer } from '$lib/window'
	import { delete_track_list, get_track_list, import_tracks, type PlaylistInfo } from '$lib/data'
	import { play_pause, playing_track, time_record } from './lib/player'
	import DragGhost from './components/DragGhost.svelte'
	import ItunesImport from './components/ItunesImport.svelte'
	import { modal_count } from './components/Modal.svelte'
	import QuickNav from './components/QuickNav.svelte'
	import { check_shortcut } from './lib/helpers'
	import ArtistList from './components/ArtistList.svelte'
	import { tracklist_actions } from './lib/page'
	import Route from './lib/Route.svelte'
	import { navigate_back, navigate_forward } from './lib/router'
	import './lib/router'
	import CheckForUpdates from './components/CheckForUpdates.svelte'
	import Settings from './components/Settings.svelte'
	import Button from './components/Button.svelte'
	import Visualizer from '$lib/visualizer/Visualizer.svelte'

	ipc_renderer.invoke('app_loaded').catch(() => {
		ipc_renderer.invoke('showMessageBox', false, {
			type: 'error',
			message: 'Failed to signal app loading',
			detail: 'Graceful shutdown will not be possible.',
		})
	})

	let media_keys_result: Awaited<ReturnType<typeof init_media_keys>> | null = $state(null)
	async function init_media_keys(prompt: boolean) {
		return await ipc_renderer.invoke('init_media_keys', prompt)
	}
	init_media_keys(false).then((result) => (media_keys_result = result))

	async function open_import_dialog() {
		if ($modal_count !== 0) {
			return
		}
		let result = await ipc_renderer.invoke('showOpenDialog', false, {
			properties: ['openFile', 'multiSelections'],
			filters: [{ name: 'Audio', extensions: ['mp3', 'm4a', 'opus'] }],
		})
		if (!result.canceled && result.filePaths.length >= 1) {
			import_tracks(result.filePaths)
		}
	}
	ipc_renderer.on('import', open_import_dialog)
	onDestroy(() => {
		ipc_renderer.removeListener('import', open_import_dialog)
	})

	function toggle_visualizer() {
		show_visualizer = !show_visualizer
	}
	ipc_renderer.on('Toggle Visualizer', toggle_visualizer)
	onDestroy(() => {
		ipc_renderer.removeListener('Toggle Visualizer', toggle_visualizer)
	})

	let droppable = $state(false)
	const allowed_mimes = ['audio/mpeg', 'audio/x-m4a', 'audio/ogg'] // mp3, m4a
	function get_file_paths(e: DragEvent): string[] {
		if (!e.dataTransfer) return []
		let valid_paths: string[] = []
		for (let i = 0; i < e.dataTransfer.files.length; i++) {
			const file = e.dataTransfer.files[i]
			if (allowed_mimes.includes(file.type)) {
				valid_paths.push(file.path)
			}
		}
		return valid_paths
	}
	function has_files(e: DragEvent): boolean {
		if (!e.dataTransfer) return false
		for (let i = 0; i < e.dataTransfer.items.length; i++) {
			const item = e.dataTransfer.items[i]
			if (item.kind === 'file' && allowed_mimes.includes(item.type)) {
				return true
			}
		}
		return false
	}
	function drag_enter_or_over(e: DragEvent) {
		droppable = has_files(e)
		if (droppable) {
			e.preventDefault()
		}
	}
	function drag_leave() {
		droppable = false
	}
	function drop(e: DragEvent) {
		e.preventDefault()
		droppable = false
		const valid_paths = get_file_paths(e)
		const paths = []
		for (const path of valid_paths) {
			paths.push(path)
		}
		import_tracks(paths)
	}
	function keydown(e: KeyboardEvent) {
		let el = e.target as HTMLAudioElement
		const space_tags = ['INPUT', 'TEXTAREA', 'BUTTON', 'SELECT']
		if (el && !space_tags.includes(el.tagName) && $modal_count === 0) {
			const prevent_el = el.closest('[data-prevent-space-play]')
			if (!prevent_el && e.key === ' ') {
				e.preventDefault()
				play_pause()
			}
		}
	}

	let show_settings = $state(false)
	onDestroy(
		ipc_listen('show_settings', () => {
			if ($modal_count === 0) {
				show_settings = true
			}
		}),
	)

	let show_itunes_import = $state(false)
	onDestroy(
		ipc_listen('itunesImport', () => {
			if ($modal_count === 0) {
				show_itunes_import = true
			}
		}),
	)

	let show_visualizer = $state(false)

	let playlist_info: PlaylistInfo | null = $state(null)
	onDestroy(
		ipc_listen('context.playlist.edit', (_, id) => {
			const list = get_track_list(id)
			if (list.type !== 'special' && $modal_count === 0) {
				playlist_info = {
					name: list.name,
					description: list.description || '',
					isFolder: list.type === 'folder',
					id: list.id,
					editMode: true,
				}
			}
		}),
	)
	onDestroy(
		ipc_listen('context.playlist.delete', async (_, id) => {
			const list = get_track_list(id)
			const result = await ipc_renderer.invoke('showMessageBox', false, {
				type: 'info',
				message: `Delete the ${list.type} "${list.name}"?`,
				detail: list.type === 'folder' ? 'This will also delete all playlists inside.' : '',
				buttons: [`Delete`, 'Cancel'],
				defaultId: 0,
			})
			if (result.response === 0) {
				delete_track_list(id)
			}
		}),
	)
	onDestroy(
		ipc_listen('newPlaylist', (_, id, is_folder) => {
			playlist_info = {
				name: '',
				description: '',
				isFolder: is_folder,
				id: id,
				editMode: false,
			}
		}),
	)

	onDestroy(ipc_listen('Back', navigate_back))
	onDestroy(ipc_listen('Forward', navigate_forward))

	onMount(() => {
		tracklist_actions.focus()
	})
</script>

<svelte:window
	onkeydown={keydown}
	onfocus={async () => {
		if (media_keys_result?.needs_accessibility_permission) {
			media_keys_result = null
			media_keys_result = await ipc_renderer.invoke('init_media_keys', false)
		}
	}}
/>
<svelte:head>
	<title>Ferrum</title>
</svelte:head>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<main
	ondragentercapture={drag_enter_or_over}
	onkeydown={(e) => {
		if (e.target) {
			if (check_shortcut(e, 'ArrowUp', { cmd_or_ctrl: true })) {
				e.preventDefault()
				ipc_renderer.invoke('volume_change', true)
			} else if (check_shortcut(e, 'ArrowDown', { cmd_or_ctrl: true })) {
				e.preventDefault()
				ipc_renderer.invoke('volume_change', false)
			}
		}
	}}
>
	<div class="meat">
		<Sidebar />
		<div class="flex size-full min-w-0 flex-col">
			<Route route="/playlist/:playlist_id" component={TrackList} />
			<Route route="/artists" component={ArtistList} />
			{#if media_keys_result}
				<div class="shrink-0 overflow-hidden">
					<div
						class="flex items-center border-t border-r border-l border-red-500/15 bg-red-500/10 px-2 py-1 text-[15px] text-red-500 select-text"
						in:fly|global={{ y: '100%', duration: 200 }}
					>
						{#if media_keys_result.error}
							{media_keys_result.error}
							<Button
								type="button"
								secondary
								thin
								onclick={async () => {
									media_keys_result = null
									media_keys_result = await ipc_renderer.invoke('init_media_keys', false)
								}}>Retry</Button
							>
							<Button
								type="button"
								secondary
								thin
								onclick={() => {
									media_keys_result = null
								}}>Ignore</Button
							>
						{:else if media_keys_result.needs_accessibility_permission}
							Do you want to Ferrum to control the media keys? You may need to delete and re-add the
							app from the accessibility list
							<Button
								class="text-nowrap"
								type="button"
								secondary
								thin
								onclick={async () => {
									media_keys_result = null
									media_keys_result = await ipc_renderer.invoke('init_media_keys', true)
								}}>Give permission</Button
							>
							<Button
								type="button"
								secondary
								thin
								onclick={() => {
									media_keys_result = null
								}}>Ignore</Button
							>
						{/if}
					</div>
				</div>
			{/if}
		</div>
		{#if $queue_visible}
			<Queue />
		{:else if lyrics_state.visible}
			<Lyrics
				track_name={$playing_track?.name || ''}
				artist_name={$playing_track?.artist || ''}
				album_name={$playing_track?.albumName || ''}
				duration_sec={$time_record.duration}
				current_time_sec={$time_record.elapsed}
			/>
		{/if}
	</div>
	<Player on_toggle_visualizer={toggle_visualizer} />
	{#if droppable}
		<!-- if the overlay is always visible, it's not possible to scroll while dragging tracks -->
		<div class="drag-overlay" transition:fade={{ duration: 100 }}>
			<h1>Drop files to import</h1>
		</div>
		<!-- svelte-ignore a11y_interactive_supports_focus -->
		<div
			class="dropzone"
			ondragleave={drag_leave}
			ondrop={drop}
			ondragover={drag_enter_or_over}
			role="dialog"
			aria-label="Drop files to import"
			aria-dropeffect="copy"
		></div>
	{/if}
</main>

{#if show_visualizer}
	<Visualizer on_close={toggle_visualizer} />
{/if}

<QuickNav />
{#if track_info_state.instance}
	<TrackInfo bind:instance={track_info_state.instance} />
{/if}
{#if playlist_info}
	<PlaylistInfoModal info={playlist_info} cancel={() => (playlist_info = null)} />
{/if}
{#if show_itunes_import}
	<ItunesImport cancel={() => (show_itunes_import = false)} />
{/if}
{#if show_settings}
	<Settings on_close={() => (show_settings = false)} />
{/if}
<CheckForUpdates />

<DragGhost />

<style lang="sass">
	:global(:root)
		--cubic-out: cubic-bezier(0.33, 1, 0.68, 1)
		--player-bg-color: #17181c
		--text-color: #e6e6e6
		--drag-bg-color: #1e1f24
		--drag-line-color: #0083f5
		--empty-cover-bg-color: #2b2c31
		--empty-cover-color: #45464a
		--border-color: #333333
		--accent-1: #2e5be0
		--accent-2: #103fcb
		--icon-color: #e6e6e6
		--icon-highlight-color: #00ffff
		--titlebar-height: 22px
		--hue: 225
		--right-sidebar-width: 250px
	:global(html), :global(body)
		background-color: #0D1115
		height: 100%
		color-scheme: dark
	:global(body)
		position: relative
		width: 100%
		margin: 0
		box-sizing: border-box
		background-image: linear-gradient(150deg, hsl(var(--hue), 60%, 10%), hsl(var(--hue), 20%, 6%))
		color: var(--text-color)
		font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen-Sans, Ubuntu, Cantarell, "Helvetica Neue", sans-serif
		user-select: none
	.dropzone, .drag-overlay
		position: fixed
		width: 100%
		height: 100%
		top: 0px
		left: 0px
	.drag-overlay
		display: flex
		align-items: center
		justify-content: center
		background-color: rgba(#10161e, 0.9)
		transition: all 100ms ease-in-out
	main
		height: 100%
		max-height: 100%
		display: flex
		flex-direction: column
	.meat
		position: relative
		height: 0px
		display: flex
		flex-direction: row
		flex-grow: 1
</style>
