<script lang="ts" module>
	import {
		track_lists_details_map,
		add_tracks_to_playlist,
		move_playlist,
		view_options,
		save_view_options,
	} from '$lib/data'
	import Self from './SidebarItems.svelte'

	export type SidebarItemHandle = {
		handleKey(e: KeyboardEvent): void
	}

	const shown_folders = writable(view_options.shownPlaylistFolders)

	function show_folder(id: string) {
		view_options.shownPlaylistFolders.push(id)
		shown_folders.set(view_options.shownPlaylistFolders)
		save_view_options(view_options)
	}
	function hide_folder(id: string) {
		view_options.shownPlaylistFolders = view_options.shownPlaylistFolders.filter(
			(folder_id) => folder_id !== id,
		)
		shown_folders.set(view_options.shownPlaylistFolders)
		save_view_options(view_options)
	}
</script>

<script lang="ts">
	import { run, stopPropagation, self, preventDefault } from 'svelte/legacy'

	import type { TrackListDetails } from '../../ferrum-addon'
	import { type Writable, writable } from 'svelte/store'
	import { getContext } from 'svelte'
	import { dragged } from '../lib/drag-drop'
	import * as dragGhost from './DragGhost.svelte'
	import { ipc_renderer } from '$lib/window'
	import { check_shortcut } from '$lib/helpers'
	import { navigate, url_pathname } from '$lib/router'
	import { current_playlist_id } from './TrackList.svelte'
	import { tracklist_actions } from '$lib/page'

	type Child = TrackListDetails

	async function tracklist_context_menu(id: string, is_folder: boolean) {
		await ipc_renderer.invoke('showTracklistMenu', { id, isFolder: is_folder, isRoot: false })
	}

	function has_showing_children(id: string) {
		const list = $track_lists_details_map[id]
		return list.children && list.children.length > 0 && $shown_folders.includes(id)
	}

	interface Props {
		show?: boolean
		parent_id: string | null
		prevent_drop?: boolean
		children: Child[]
		level?: number
		on_select_down?: any
	}

	let {
		show = true,
		parent_id,
		prevent_drop = false,
		children,
		level = 0,
		on_select_down = () => {},
	}: Props = $props()
	function select_first(item: Child) {
		const child_id = item.children?.[0]
		if (child_id) {
			navigate('/playlist/' + $track_lists_details_map[child_id].id)
		}
	}
	function select_last(in_id: string) {
		const children = $track_lists_details_map[in_id].children
		if (children && (has_showing_children(in_id) || in_id === 'root')) {
			select_last(children[children.length - 1])
		} else {
			navigate('/playlist/' + $track_lists_details_map[in_id].id)
		}
	}
	function select_up(i: number) {
		const prev = children[i - 1] || null
		if (i === 0 && parent_id) {
			navigate(`/playlist/${parent_id}`)
		} else if (prev && has_showing_children(prev.id)) {
			select_last(prev.id)
		} else if (prev) {
			navigate(`/playlist/${prev.id}`)
		}
	}
	function select_down(i: number) {
		if (has_showing_children(children[i].id)) {
			select_first(children[i])
		} else if (children[i + 1]) {
			navigate(`/playlist/${children[i + 1].id}`)
		} else {
			on_select_down()
		}
	}

	export function handle_key(e: KeyboardEvent) {
		const index = children.findIndex((child) => {
			return `/playlist/${child.id}` === $url_pathname
		})
		if (index < 0) {
			return
		}

		const selected_list = $track_lists_details_map[$current_playlist_id]
		if (check_shortcut(e, 'ArrowUp')) {
			select_up(index)
		} else if (check_shortcut(e, 'ArrowUp', { alt: true })) {
			navigate('/playlist/root')
		} else if (check_shortcut(e, 'ArrowDown', { alt: true })) {
			select_last('root')
		} else if (check_shortcut(e, 'ArrowDown')) {
			select_down(index)
		} else if (check_shortcut(e, 'ArrowLeft')) {
			if (selected_list.kind === 'folder' && $shown_folders.includes(selected_list.id)) {
				hide_folder(selected_list.id)
			} else if (parent_id) {
				navigate(`/playlist/${parent_id}`)
			}
		} else if (check_shortcut(e, 'ArrowRight') && selected_list.kind === 'folder') {
			show_folder(selected_list.id)
		} else {
			return
		}
		e.preventDefault()
	}
	run(() => {
		if (children.find((child) => `/playlist/${child.id}` === $url_pathname)) {
			const item_handle = getContext<Writable<SidebarItemHandle | null>>('itemHandle')
			item_handle.set({ handleKey: handle_key })
		}
	})

	let drag_track_onto_index = $state(null as number | null)
	let drop_above = $state(false)
	let drag_playlist_onto_index = $state(null as number | null)

	function on_drag_start(e: DragEvent, tracklist: TrackListDetails) {
		if (e.dataTransfer && tracklist.kind !== 'special' && parent_id) {
			e.dataTransfer.clearData()
			e.dataTransfer.effectAllowed = 'move'
			dragGhost.set_inner_text(tracklist.name)
			dragged.playlist = {
				id: tracklist.id,
				from_folder: parent_id,
				level,
			}
			e.dataTransfer.setDragImage(dragGhost.drag_el, 0, 0)
			e.dataTransfer.setData('ferrum.playlist', '')
		}
	}
</script>

<!-- Don't propagate mousedown, to prevent dragging from breaking -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
	class="sub"
	class:show
	onmousedown={stopPropagation(() => {
		requestAnimationFrame(() => {
			tracklist_actions.focus()
		})
	})}
>
	{#each children as child_list, i}
		{#if child_list.kind === 'folder'}
			<a
				href="/playlist/{child_list.id}"
				tabindex="-1"
				class="item rounded-r-[5px] outline-none"
				style:padding-left={14 * level + 'px'}
				class:active={`/playlist/${child_list.id}` === $url_pathname}
				draggable="true"
				ondragstart={(e) => on_drag_start(e, child_list)}
				class:show={$shown_folders.includes(child_list.id)}
				class:droppable={drag_playlist_onto_index === i}
				ondrop={(e) => {
					if (
						e.currentTarget &&
						e.dataTransfer?.types[0] === 'ferrum.playlist' &&
						dragged.playlist &&
						!prevent_drop &&
						dragged.playlist.id !== child_list.id &&
						child_list.children !== undefined
					) {
						move_playlist(
							dragged.playlist.id,
							dragged.playlist.from_folder,
							child_list.id,
							Math.max(0, child_list.children.length - 1),
						)
						drag_playlist_onto_index = null
					}
				}}
				onmousedown={() => navigate(`/playlist/${child_list.id}`)}
				oncontextmenu={() => tracklist_context_menu(child_list.id, true)}
			>
				<!-- svelte-ignore a11y_interactive_supports_focus -->
				<svg
					class="arrow"
					role="button"
					aria-label="Arrow button"
					onmousedown={stopPropagation(() => {
						if ($shown_folders.includes(child_list.id)) {
							hide_folder(child_list.id)
						} else {
							show_folder(child_list.id)
						}
					})}
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="0 0 24 24"
				>
					<path d="M21 12l-18 12v-24z" />
				</svg>
				<!-- svelte-ignore a11y_interactive_supports_focus -->
				<div
					class="text"
					role="link"
					ondragover={(e) => {
						if (
							e.currentTarget &&
							e.dataTransfer?.types[0] === 'ferrum.playlist' &&
							dragged.playlist &&
							!prevent_drop &&
							dragged.playlist.id !== child_list.id
						) {
							drag_playlist_onto_index = i
							e.preventDefault()
						}
					}}
					ondragleave={self(() => {
						drag_playlist_onto_index = null
					})}
				>
					{child_list.name}
				</div>
			</a>
			<Self
				show={$shown_folders.includes(child_list.id)}
				parent_id={child_list.id}
				children={($track_lists_details_map[child_list.id].children || []).map(
					(child_id) => $track_lists_details_map[child_id],
				)}
				level={level + 1}
				prevent_drop={prevent_drop || dragged.playlist?.id === child_list.id}
				on_select_down={() => {
					if (i < children.length - 1) {
						navigate(`/playlist/${children[i + 1].id}`)
					} else {
						on_select_down()
					}
				}}
			/>
		{:else if child_list.kind === 'playlist'}
			<!-- svelte-ignore a11y_interactive_supports_focus -->
			<a
				href="/playlist/{child_list.id}"
				tabindex="-1"
				class="item rounded-r-[5px]"
				aria-label="playlist"
				style:padding-left={14 * level + 'px'}
				draggable="true"
				ondragstart={(e) => on_drag_start(e, child_list)}
				class:active={`/playlist/${child_list.id}` === $url_pathname}
				onmousedown={() => navigate(`/playlist/${child_list.id}`)}
				class:droppable={drag_track_onto_index === i}
				class:droppable-above={drag_playlist_onto_index === i && drop_above}
				class:droppable-below={drag_playlist_onto_index === i && !drop_above}
				ondrop={(e) => {
					if (e.currentTarget && e.dataTransfer?.types[0] === 'ferrum.tracks' && dragged.tracks) {
						add_tracks_to_playlist(child_list.id, dragged.tracks.ids)
						drag_track_onto_index = null
					} else if (
						e.currentTarget &&
						e.dataTransfer?.types[0] === 'ferrum.playlist' &&
						dragged.playlist &&
						!prevent_drop &&
						parent_id !== null
					) {
						const rect = e.currentTarget.getBoundingClientRect()
						drop_above = e.pageY < rect.bottom - rect.height / 2
						move_playlist(
							dragged.playlist.id,
							dragged.playlist.from_folder,
							parent_id,
							drop_above ? i : i + 1,
						)
						drag_playlist_onto_index = null
					}
				}}
				oncontextmenu={() => tracklist_context_menu(child_list.id, false)}
			>
				<div class="arrow"></div>
				<div
					class="text"
					role="link"
					ondragover={(e) => {
						if (e.currentTarget && e.dataTransfer?.types[0] === 'ferrum.tracks' && dragged.tracks) {
							drag_track_onto_index = i
							e.preventDefault()
						} else if (
							e.currentTarget &&
							e.dataTransfer?.types[0] === 'ferrum.playlist' &&
							dragged.playlist &&
							!prevent_drop
						) {
							drag_playlist_onto_index = i
							e.preventDefault()
							const rect = e.currentTarget.getBoundingClientRect()
							drop_above = e.pageY < rect.bottom - rect.height / 2
						}
					}}
					ondragleave={self(() => {
						drag_track_onto_index = null
						drag_playlist_onto_index = null
					})}
				>
					{child_list.name}
				</div>
			</a>
		{:else}
			<a
				href="/playlist/{child_list.id}"
				tabindex="-1"
				class="item rounded-r-[5px]"
				style:padding-left={14 * level + 'px'}
				onmousedown={preventDefault(() => navigate(`/playlist/${child_list.id}`))}
				class:active={`/playlist/${child_list.id}` === $url_pathname}
			>
				<div class="arrow"></div>
				<div class="text">
					{child_list.name}
				</div>
			</a>
		{/if}
	{/each}
</div>

<style lang="sass">
	.item.active
		box-shadow: inset 2px 0px 0px 0px hsl(var(--hue), 70%, 60%)
		background-image: linear-gradient(90deg, hsl(var(--hue), 45%, 30%) 30%, transparent 66.6666%)
		background-position: 0% 0%
	:global(aside :focus)
		.active
			box-shadow: inset 2px 0px 0px 0px hsl(var(--hue), 70%, 60%)
			background-image: linear-gradient(90deg, hsl(var(--hue), 70%, 46%) 30%, transparent 66.6666%)
	.item
		display: flex
		align-items: center
		margin-right: 10px
		box-sizing: border-box
		z-index: 1
		background-position: 100% 0%
		background-size: 150% 150%
		transition: 260ms background-position cubic-bezier(0, 0.02, 0.2, 1)
	.item.droppable .text
		border-radius: 6px
		box-shadow: inset 0px 0px 0px 2px var(--accent-1)
		background-color: hsla(var(--hue), 74%, 53%, 0.25)
	.item.droppable-above .text
		box-shadow: 0px -1px 0px 0px var(--accent-1), inset 0px 1px 0px 0px var(--accent-1)
	.item.droppable-below .text
		box-shadow: 0px 1px 0px 0px var(--accent-1), inset 0px -1px 0px 0px var(--accent-1)
	.sub
		display: none
		&.show
			display: block
	.arrow
		margin-left: 2px
		margin-right: -8px
		padding: 6px
		width: 19px
		height: 19px
		z-index: 1
		flex-shrink: 0
		fill: white
		transition: 140ms transform var(--cubic-out)
	.show > svg.arrow
		transform: rotate(90deg)
	.text
		white-space: nowrap
		overflow: hidden
		text-overflow: ellipsis
		padding-right: 10px
		padding-left: 8px
		height: 24px
		line-height: 24px
		flex-grow: 1
		position: relative
</style>
