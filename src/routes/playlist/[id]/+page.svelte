<script lang="ts" module>
	export const sort_key = writable('index')
	export const sort_desc = writable(true)
	export let current_playlist_id = writable('')
	current_playlist_id.subscribe((id) => {
		// Cannot call get_default_sort_desc() here because it
		// would be called before addon gets initialized
		if (id === 'root') {
			sort_key.set('dateAdded')
			sort_desc.set(true)
		} else {
			sort_key.set('index')
			sort_desc.set(true)
		}
	})
	export const group_album_tracks = writable(true)
	export const tracks_page_item_ids = writable<TracksPage['itemIds']>([])
</script>

<script lang="ts">
	import {
		filter,
		move_tracks,
		remove_from_playlist,
		tracklist_updated,
		get_default_sort_desc,
		delete_tracks_with_item_ids,
		tracks_updated,
		get_tracks_page,
		get_track_ids,
		save_view_options,
		get_track_by_item_id,
		view_options,
		paths,
		join_paths,
	} from '$lib/data'
	import { new_playback_instance, playing_id } from '$lib/player'
	import { get_duration, format_date, check_mouse_shortcut, check_shortcut } from '$lib/helpers'
	import { tracklist_actions } from '$lib/page'
	import { ipc_listen, ipc_renderer } from '$lib/window'
	import { onDestroy, onMount } from 'svelte'
	import { dragged } from '$lib/drag-drop'
	import * as DragGhost from '$components/DragGhost.svelte'
	import type { ItemId, Track, TracksPage } from 'ferrum-addon'
	import Header from '$components/Header.svelte'
	import { writable } from 'svelte/store'
	import { SvelteSelection } from '$lib/selection'
	import {
		get_show_in_playlists_tree,
		get_tracklists_tree,
		handle_selected_tracks_action,
	} from '$lib/menus'
	import type { SelectedTracksAction } from '$electron/typed_ipc'
	import { RefreshLevel, VirtualGrid, type Column } from '$lib/virtual-grid.svelte'

	let tracklist_element: HTMLDivElement | undefined = $state()

	let { params } = $props()
	$effect(() => {
		$current_playlist_id = params.id
	})

	const page_options = $derived({
		playlistId: params.id,
		filterQuery: $filter,
		sortKey: $sort_key,
		sortDesc: $sort_desc,
		groupAlbumTracks: $group_album_tracks,
	})

	let tracks_page = $derived(get_tracks_page(page_options))
	onDestroy(
		tracklist_updated.subscribe(() => {
			tracks_page = get_tracks_page(page_options)
		}),
	)
	onDestroy(
		tracks_updated.subscribe(() => {
			tracks_page = get_tracks_page(page_options)
		}),
	)
	$effect(() => {
		$tracks_page_item_ids = tracks_page.itemIds
	})

	function handle_action(action: SelectedTracksAction) {
		if (selection.items.size === 0) {
			return
		} else if (action === 'Remove from Playlist') {
			remove_from_playlist($current_playlist_id, selection.items_as_array())
			return
		} else if (action === 'Delete from Library') {
			delete_tracks(selection.items_as_array())
		} else {
			handle_selected_tracks_action({
				action,
				track_ids: get_track_ids(selection.items_as_array()),
				all_ids: get_track_ids(tracks_page.itemIds),
				first_index: selection.find_first_index(),
			})
		}
	}

	// svelte-ignore state_referenced_locally
	const selection = new SvelteSelection(tracks_page.itemIds, {
		scroll_to({ index }) {
			tracklist_actions.scroll_to_index?.(index)
		},
		async on_contextmenu() {
			const selected_track_ids = get_track_ids(selection.items_as_array())
			const action = await ipc_renderer.invoke('show_tracks_menu', {
				is_editable_playlist: tracks_page.playlistKind === 'playlist',
				queue: false,
				lists: get_tracklists_tree(),
				show_in_playlists: get_show_in_playlists_tree(selected_track_ids),
			})
			if (action !== null) {
				handle_action(action)
			}
		},
	})
	$effect(() => {
		selection.update_all_items(tracks_page.itemIds)
	})

	const track_action_unlisten = ipc_listen('selected_tracks_action', (_, action) => {
		if (tracklist_element?.contains(document.activeElement)) {
			handle_action(action)
		}
	})
	onDestroy(track_action_unlisten)

	ipc_renderer.on('Group Album Tracks', (_, checked) => {
		group_album_tracks.set(checked)
	})

	function double_click(e: MouseEvent, index: number) {
		if (e.button === 0 && check_mouse_shortcut(e)) {
			play_row(index)
		}
	}
	async function delete_tracks(item_ids: ItemId[]) {
		const s = selection.items.size > 1 ? 's' : ''
		const result = await ipc_renderer.invoke('showMessageBox', false, {
			type: 'info',
			message: `Delete ${selection.items.size} song${s} from library?`,
			buttons: [`Delete Song${s}`, 'Cancel'],
			defaultId: 0,
		})
		if (result.response === 0) {
			delete_tracks_with_item_ids(item_ids)
		}
	}
	async function keydown(e: KeyboardEvent) {
		if (check_shortcut(e, 'Enter')) {
			let first_item_id = selection.find_first_index()
			if (first_item_id !== null) {
				play_row(first_item_id)
			} else if (!$playing_id) {
				play_row(0)
			}
		} else if (
			check_shortcut(e, 'Backspace') &&
			selection.items.size > 0 &&
			$filter === '' &&
			tracks_page.playlistKind === 'playlist'
		) {
			e.preventDefault()
			const s = selection.items.size > 1 ? 's' : ''
			const result = ipc_renderer.invoke('showMessageBox', false, {
				type: 'info',
				message: `Remove ${selection.items.size} song${s} from the list?`,
				buttons: ['Remove Song' + s, 'Cancel'],
				defaultId: 0,
			})
			if ((await result).response === 0) {
				remove_from_playlist(params.id, Array.from(selection.items))
			}
		} else if (check_shortcut(e, 'Backspace', { cmd_or_ctrl: true }) && $selection.size > 0) {
			e.preventDefault()
			handle_action('Delete from Library')
		} else {
			selection.handle_keydown(e)
			return
		}
		e.preventDefault()
	}

	function play_row(index: number) {
		const all_track_ids = get_track_ids(tracks_page.itemIds)
		new_playback_instance(all_track_ids, index)
	}

	let drag_line: HTMLElement | undefined = $state()
	let drag_item_ids: ItemId[] = []
	function on_drag_start(e: DragEvent) {
		if (e.dataTransfer) {
			drag_item_ids = Array.from(selection.items)
			e.dataTransfer.effectAllowed = 'move'
			if (drag_item_ids.length === 1) {
				const { track } = get_track_by_item_id(drag_item_ids[0])
				DragGhost.set_inner_text(track.artist + ' - ' + track.name)
			} else {
				DragGhost.set_inner_text(drag_item_ids.length + ' items')
			}
			dragged.tracks = {
				ids: get_track_ids(drag_item_ids),
				playlist_indexes: drag_item_ids,
			}
			e.dataTransfer.setDragImage(DragGhost.drag_el, 0, 0)
			e.dataTransfer.setData('ferrum.tracks', '')
		}
	}

	let viewport: HTMLElement | undefined = $state()
	let drag_to_index: null | number = $state(null)
	function on_drag_over(e: DragEvent, element: Element, index: number) {
		if (
			!$sort_desc ||
			$sort_key !== 'index' ||
			$filter ||
			tracks_page.playlistKind !== 'playlist'
		) {
			drag_to_index = null
			return
		}
		if (
			dragged.tracks?.playlist_indexes &&
			e.currentTarget instanceof HTMLElement &&
			e.dataTransfer?.types[0] === 'ferrum.tracks' &&
			drag_line &&
			viewport
		) {
			e.preventDefault()
			const rect = element.getBoundingClientRect()
			const container_rect = viewport.getBoundingClientRect()
			if (e.pageY < rect.bottom - rect.height / 2) {
				const top = rect.top - container_rect.top + viewport.scrollTop - 1
				drag_line.style.top = Math.max(0, top) + 'px'
				drag_to_index = index
			} else {
				const top = rect.bottom - container_rect.top + viewport.scrollTop - 1
				const max_top = viewport.scrollHeight - 2
				drag_line.style.top = Math.min(max_top, top) + 'px'
				drag_to_index = index + 1
			}
		}
	}
	function drop_handler() {
		if (drag_to_index !== null) {
			const _result = move_tracks(params.id, drag_item_ids, drag_to_index)
			// Same handling for success and error
			drag_to_index = null
		}
	}

	function get_item(item_id: ItemId) {
		try {
			return get_track_by_item_id(item_id)
		} catch (_) {
			return { id: null, track: null }
		}
	}

	type TrackListColumn = Column & {
		name: string
		key: 'index' | 'image' | keyof Track
		filter?: string | undefined
	}
	const all_columns: TrackListColumn[] = [
		// sorted alphabetically
		{ name: '#', key: 'index', width: 46 },
		// { name: 'Size', key: 'size' },
		{ name: 'Album', key: 'albumName', filter: 'album', width: 0.9, is_pct: true },
		{ name: 'Album Artist', key: 'albumArtist', filter: 'albumartist', width: 0.9, is_pct: true },
		{ name: 'Artist', key: 'artist', filter: 'artist', width: 1.2, is_pct: true },
		// { name: 'Bitrate', key: 'bitrate' },
		{ name: 'BPM', key: 'bpm', filter: 'bpm', width: 43 },
		{ name: 'Comments', key: 'comments', filter: 'comment', width: 0.65, is_pct: true },
		// { name: 'Compilation', key: 'compilation' },
		{ name: 'Composer', key: 'composer', filter: 'composer', width: 0.65, is_pct: true },
		{ name: 'Date Added', key: 'dateAdded', width: 140 },
		// { name: 'DateImported', key: 'dateImported' },
		// { name: 'DateModified', key: 'dateModified' },
		// { name: 'Disabled', key: 'disabled' },
		// { name: 'DiscCount', key: 'discCount' },
		// { name: 'DiscNum', key: 'discNum' },
		// { name: 'Disliked', key: 'disliked' },
		{ name: 'Time', key: 'duration', width: 50 },
		{ name: 'Genre', key: 'genre', filter: 'genre', width: 0.65, is_pct: true },
		{ name: 'Grouping', key: 'grouping', filter: 'grouping', width: 0.65, is_pct: true },
		{
			name: 'Image',
			key: 'image',
			width: 28,
			cell_render(cell, value) {
				if (typeof value !== 'string') {
					throw new Error('Non-string image value')
				}
				if (cell.firstChild instanceof HTMLImageElement && cell.firstChild.src === value) {
					return
				}
				let img: HTMLImageElement
				if (cell.firstChild instanceof HTMLImageElement) {
					img = cell.firstChild
				} else {
					img = document.createElement('img')
				}
				img.classList.add('invisible')
				img.src = value
				img.onload = (e) => {
					const img = e.currentTarget as HTMLImageElement
					if (img.src !== value) return
					img.classList.remove('invisible', 'error', 'missing')
					img.removeAttribute('title')
				}
				img.onerror = async (e) => {
					// Yes this is dumb, but there's no way to get an error code from <img src="" />
					const img = (e as Event).currentTarget as HTMLImageElement
					if (img.src !== value) return
					img.classList.remove('invisible')
					img.removeAttribute('title')
					// 404 is an common expected result, so we start with that
					img.classList.add('error', 'missing')
					const new_error = await fetch(value)
						.then(async (response) => {
							if (response.status === 404) {
								return '404'
							}
							const response_text = await response.text()
							console.log(`Failed to load cover (${response.status}): ${response_text}`)
							return response_text
						})
						.catch(() => {
							return 'network'
						})
					if (img.src !== value) return
					img.classList.toggle('missing', new_error === '404')
					img.title = new_error
				}
				img.onmousedown = (e) => {
					const img = e.currentTarget as HTMLImageElement
					if (img.classList.contains('error')) {
						e.stopPropagation()
					}
				}
				img.onclick = (e) => {
					const img = e.currentTarget as HTMLImageElement
					const error = (e.currentTarget as HTMLImageElement).title
					if (img.classList.contains('error')) {
						ipc_renderer.invoke('showMessageBox', false, {
							type: 'error',
							message: 'Failed to load cover',
							detail: error,
						})
					}
				}
				cell.replaceChildren(img)
			},
		},
		// { name: 'ImportedFrom', key: 'importedFrom' },
		// { name: 'Liked', key: 'liked' },
		{ name: 'Name', key: 'name', filter: 'name', width: 1.7, is_pct: true },
		{ name: 'Plays', key: 'playCount', filter: 'plays', width: 52 },
		// { name: 'Rating', key: 'rating' },
		// { name: 'SampleRate', key: 'sampleRate' },
		{ name: 'Skips', key: 'skipCount', filter: 'skips', width: 52 },
		// { name: 'Sort Album', key: 'sortAlbumName', width: 0.65, is_pct: true },
		// { name: 'Sort Album Artist', key: 'sortAlbumArtist', width: 0.65, is_pct: true },
		// { name: 'Sort Artist', key: 'sortArtist', width: 0.65, is_pct: true },
		// { name: 'Sort Composer', key: 'sortComposer', width: 0.65, is_pct: true },
		// { name: 'Sort Name', key: 'sortName', width: 0.65, is_pct: true },
		// { name: 'TrackCount', key: 'trackCount' },
		// { name: 'TrackNum', key: 'trackNum' },
		// { name: 'Volume', key: 'volume' },
		{ name: 'Year', key: 'year', filter: 'year', width: 47 },
	]
	const default_columns: Column['key'][] = [
		'index',
		'image',
		'name',
		'playCount',
		'duration',
		'artist',
		'albumName',
		'comments',
		'genre',
		'dateAdded',
		'year',
	]
	let columns: Column[] = $state(load_columns())
	onMount(() => {
		columns = load_columns()
	})
	function load_columns(): Column[] {
		let loaded_columns = view_options.columns
		if (loaded_columns.length === 0) {
			loaded_columns = [...default_columns]
		}
		const columns = loaded_columns
			.map((key) => all_columns.find((col) => col.key === key))
			.filter((col) => col !== undefined)
		return columns
	}
	function save_columns() {
		view_options.columns = columns.map((col) => col.key)
		if (JSON.stringify(view_options.columns) === JSON.stringify(default_columns)) {
			view_options.columns = []
		}
		save_view_options(view_options)
	}
	onDestroy(
		ipc_listen('context.toggle_column', (_, item) => {
			if (item.checked) {
				const column = all_columns.find((column) => column.key === item.id)
				if (column) {
					columns.push({ ...column })
					columns = columns
					save_columns()
				}
			} else {
				columns = columns.filter((column) => column.key !== item.id)
				save_columns()
			}
		}),
	)

	let col_container: HTMLElement | undefined = $state()
	let col_drag_line: HTMLElement | undefined = $state()
	let col_drag_index: number | null = null
	function on_col_drag_start(e: DragEvent, index: number) {
		if (e.dataTransfer) {
			col_drag_index = index
		}
	}
	let col_drag_to_index: null | number = $state(null)
	function on_col_drag_over(e: DragEvent, index: number) {
		if (
			col_drag_index !== null &&
			e.currentTarget instanceof HTMLElement &&
			col_container &&
			col_drag_line
		) {
			e.preventDefault()
			const rect = e.currentTarget.getBoundingClientRect()
			const container_rect = col_container.getBoundingClientRect()
			if (e.pageX < rect.right - rect.width / 2) {
				const offset = index === 0 ? 0 : -1
				col_drag_line.style.left = rect.left - container_rect.left + offset + 'px'
				col_drag_to_index = index
			} else {
				const offset = index === columns.length - 1 ? -2 : -1
				col_drag_line.style.left = rect.right - container_rect.left + offset + 'px'
				col_drag_to_index = index + 1
			}
		}
	}
	async function col_drop_handler() {
		if (col_drag_index !== null && col_drag_to_index !== null) {
			const move_to_left = col_drag_index < col_drag_to_index
			const removed_column = columns.splice(col_drag_index, 1)[0]
			columns.splice(col_drag_to_index - Number(move_to_left), 0, removed_column)
			columns = columns
			save_columns()

			col_drag_to_index = null
		}
	}
	function col_drag_end_handler() {
		col_drag_to_index = null
	}

	function get_row(e: Event) {
		if (!(e.target instanceof Element)) {
			return null
		}
		const row = e.target?.closest('[aria-rowindex]')
		const row_number = parseInt(row?.getAttribute('aria-rowindex') ?? '')
		if (!row || !Number.isInteger(row_number)) {
			return null
		}
		return {
			index: row_number - 1,
			element: row,
		}
	}

	// svelte-ignore state_referenced_locally
	const virtual_grid = VirtualGrid.create(tracks_page.itemIds, {
		buffer: 20,
		row_prepare(item_id, i) {
			const { track, id } = get_item(item_id)
			if (track === null) {
				throw new Error(`Track with item_id ${item_id} not found`)
			}
			return {
				...track,
				item_id,
				track_id: id,
				duration: track.duration ? get_duration(track.duration) : '',
				dateAdded: format_date(track.dateAdded),
				index: i + 1,
				// Use new URL() to get the same URL as img.src uses. For example, this
				// encodes `'`, unline encodeURIComponent. This prevents
				// unecessary image reloading.
				image: new URL(
					'app://trackimg/?path=' +
						encodeURIComponent(join_paths(paths.tracksDir, track.file)) +
						'&cache_db_path=' +
						encodeURIComponent(paths.cacheDb) +
						'&date_modified=' +
						track.dateModified,
				).href,
			}
		},
		row_render(row, item, i) {
			row.setAttribute('draggable', 'true')
			row.classList.toggle('odd', i % 2 === 0)
			row.classList.toggle('selected', $selection.has(item.item_id))
			row.classList.toggle('playing', !!$playing_id && $playing_id === item.track_id)
		},
	})
	$effect(() => {
		virtual_grid.set_columns(columns)
	})
	$effect(() => {
		virtual_grid.set_source_items(tracks_page.itemIds)
	})
	onDestroy(
		selection.subscribe(() => {
			virtual_grid.refresh(RefreshLevel.AllRows)
		}),
	)
	onDestroy(
		playing_id.subscribe(() => {
			virtual_grid.refresh(RefreshLevel.AllRows)
		}),
	)

	onMount(() => {
		tracklist_actions.scroll_to_index = virtual_grid.scroll_to_index.bind(virtual_grid)
		tracklist_actions.go_to_index = (index) => {
			virtual_grid.scroll_to_index(index)
			selection.clear()
			selection.add_index(index)
		}
	})
</script>

<Header
	title={params.id === 'root' ? 'Songs' : tracks_page.playlistName}
	subtitle="{tracks_page.itemIds.length} songs"
	description={tracks_page.playlistDescription}
/>
<div
	bind:this={tracklist_element}
	class="tracklist h-full"
	role="table"
	ondragleave={() => (drag_to_index = null)}
>
	<!-- svelte-ignore a11y_interactive_supports_focus -->
	<div
		class="row table-header shrink-0 border-b border-b-slate-500/30"
		class:desc={$sort_desc}
		role="row"
		ondragleave={() => (col_drag_to_index = null)}
		bind:this={col_container}
	>
		{#each virtual_grid.columns as column, i}
			<!-- svelte-ignore a11y_interactive_supports_focus -->
			<!-- svelte-ignore a11y_click_events_have_key_events -->
			<div
				class="cell {column.key}"
				class:sort={$sort_key === column.key}
				style:width="{column.width}px"
				style:translate="{column.offset}px 0"
				role="button"
				onclick={() => {
					if (tracks_page.playlistKind === 'special' && column.key === 'index') {
						return
					} else if (column.key === 'image') {
						return
					}
					if ($sort_key === column.key) {
						sort_desc.set(!$sort_desc)
					} else {
						sort_key.set(column.key)
						sort_desc.set(get_default_sort_desc(column.key))
					}
				}}
				oncontextmenu={() => {
					const column_filter = 'filter' in column ? column.filter : null
					ipc_renderer.invoke('show_columns_menu', {
						column_filter: typeof column_filter === 'string' ? column_filter : null,
						menu: all_columns.map((col) => {
							return {
								id: col.key,
								label: col.name,
								type: 'checkbox',
								checked: !!columns.find((c) => c.key === col.key),
							} as const
						}),
					})
				}}
				draggable="true"
				ondragstart={(e) => on_col_drag_start(e, i)}
				ondragend={col_drag_end_handler}
				ondragover={(e) => on_col_drag_over(e, i)}
				ondrop={col_drop_handler}
			>
				<span>{column.key === 'image' ? '' : column.name}</span>
			</div>
		{/each}
		<div
			class="col-drag-line"
			class:hidden={col_drag_to_index === null}
			bind:this={col_drag_line}
		></div>
	</div>
	<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		bind:this={viewport}
		class="main-focus-element relative h-full overflow-y-auto outline-none"
		tabindex="0"
		onmousedown={(e) => {
			if (e.target === e.currentTarget) {
				selection.clear()
			} else {
				const row = get_row(e)
				if (row) {
					selection.handle_mousedown(e, row.index)
				}
			}
		}}
		onkeydown={keydown}
		onclick={(e: MouseEvent) => {
			const row = get_row(e)
			if (row) {
				selection.handle_click(e, row.index)
			}
		}}
		ondblclick={(e: MouseEvent) => {
			const row = get_row(e)
			if (row) {
				double_click(e, row.index)
			}
		}}
		oncontextmenu={(e: MouseEvent) => {
			const row = get_row(e)
			if (row) {
				selection.handle_contextmenu(e, row.index)
			}
		}}
		ondragstart={on_drag_start}
		ondragover={(e: DragEvent) => {
			const row = get_row(e)
			if (row) {
				on_drag_over(e, row.element, row.index)
			}
		}}
		ondrop={drop_handler}
		ondragend={() => (drag_to_index = null)}
		ondragleave={() => (drag_to_index = null)}
	>
		<div {@attach virtual_grid.attach()}>
			<div class="drag-line" class:hidden={drag_to_index === null} bind:this={drag_line}></div>
		</div>
	</div>
</div>

<style lang="sass">
	.tracklist :global
		.odd
			background-color: hsla(0, 0%, 90%, 0.06)
		.selected
			background-color: hsla(var(--hue), 20%, 42%, 0.8)
	:global(:focus) .tracklist :global, .main-focus-element:focus :global
		.selected
			background-color: hsla(var(--hue), 70%, 46%, 1)
	.tracklist :global
		contain: strict
		display: flex
		flex-direction: column
		min-width: 0px
		width: 100%
		background-color: rgba(0, 0, 0, 0.01)
		overflow: hidden
		.row.table-header
			position: relative
		.cell
			overflow: visible
		.cell.sort span
			font-weight: 500
		.cell.sort span::after
			content: '▲'
			padding-left: 1px
			scale: 0.8 0.5
			display: inline-block
		.desc .cell.sort span::after
			content: '▼'
		$row-height: 24px
		.row
			contain: strict
			width: 100%
			height: $row-height
			font-size: 12px
			line-height: $row-height
			box-sizing: border-box
			position: absolute
			&.playing.selected
				color: #ffffff
			&.playing
				color: #00ffff
		.cell
			display: block
			height: 100%
			position: absolute
			vertical-align: top
			white-space: nowrap
			overflow: hidden
			text-overflow: ellipsis
			padding-left: 5px
			padding-right: 5px
			&:first-child
				padding-left: 10px
			&:last-child
				padding-right: 0px
			&.index, &.playCount, &.skipCount, &.duration
				padding-left: 0px
				padding-right: 10px
				text-align: right
			&.image:first-child
				padding-left: 7px
				padding-right: 3px
		.playing > .index
			background-repeat: no-repeat
			background-size: 16px
			background-position-y: center
			background-position-x: calc(100% - 10px)
			color: transparent
			// We use background-image because setting it with innerHTML breaks row recycling updates
			// #00ffff
			background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='%2300ffff'%3E%3Cpath d='M0 0h24v24H0z' fill='none'/%3E%3Cpath d='M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z'/%3E%3C/svg%3E")
		.playing.selected > .index
			// #ffffff
			background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='%23ffffff'%3E%3Cpath d='M0 0h24v24H0z' fill='none'/%3E%3Cpath d='M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z'/%3E%3C/svg%3E")
		.image
			display: flex
			align-items: center
			img
				display: block
				object-fit: contain
				height: 24px
				width: 18px
				padding: 3px 0
			.error::before
				content: ''
				cursor: pointer
				display: block
				width: 100%
				height: 100%
				background-image: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 18 18" fill="%23ef4444"><path d="M9,4c-2.76,0 -5,2.24 -5,5c0,2.76 2.24,5 5,5c2.76,0 5,-2.24 5,-5c0,-2.76 -2.24,-5 -5,-5Zm0.5,7.5l-1,0l0,-1l1,0l0,1Zm0,-2l-1,0l0,-3l1,0l0,3Z"/></svg>')
			.error.missing::before
				cursor: default
				background-image: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 18 18"><rect x="0" y="0" width="18" height="18" fill="%232b2c31" rx="2" ry="2"/><path fill="%2345464a" d="M12.667,5l-5.332,1.195l-0,4.347c-0.993,-0.197 -2.002,0.557 -2.002,1.384c0,0.713 0.557,1.074 1.162,1.074c0.718,-0 1.504,-0.509 1.505,-1.546l0,-3.633l4,-0.82l0,2.875c-0.992,-0.196 -2,0.554 -2,1.38c0,0.714 0.572,1.077 1.174,1.077c0.713,0 1.492,-0.508 1.493,-1.545l-0,-5.788Z"/></svg>')
		.dateAdded
			font-variant-numeric: tabular-nums
	.drag-line
		position: absolute
		width: 100%
		height: 2px
		background-color: var(--drag-line-color)
		pointer-events: none
	.col-drag-line
		position: absolute
		width: 2px
		height: 100vh
		background-color: var(--drag-line-color)
		pointer-events: none
		z-index: 5
</style>
