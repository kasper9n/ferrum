<script lang="ts" module>
	export const special_playlists_nav = [
		{ id: 'root', name: 'Songs', kind: 'special' },
		// { id: 'root', name: 'Artists', kind: 'special', path: '/artists' },
	]
</script>

<script lang="ts">
	import SidebarItems, { type SidebarItemHandle } from './SidebarItems.svelte'
	import Filter from './Filter.svelte'
	import { is_mac, track_lists_details_map, move_playlist } from '$lib/data'
	import { ipc_listen, ipc_renderer } from '../lib/window'
	import { writable } from 'svelte/store'
	import { onDestroy, setContext, tick } from 'svelte'
	import { dragged } from '../lib/drag-drop'
	import { tracklist_actions } from '$lib/page'
	import { navigate } from '$lib/router'
	import { current_playlist_id } from './TrackList.svelte'
	import { prevent_default_self } from '$lib/helpers'

	let viewport: HTMLElement | undefined = $state()
	const item_handle = setContext('itemHandle', writable(null as SidebarItemHandle | null))

	onDestroy(
		ipc_listen('Select Previous List', () => {
			$item_handle?.handleKey(new KeyboardEvent('keydown', { key: 'ArrowUp' }))
		}),
	)
	onDestroy(
		ipc_listen('Select Next List', () => {
			$item_handle?.handleKey(new KeyboardEvent('keydown', { key: 'ArrowDown' }))
		}),
	)

	async function on_context_menu() {
		await ipc_renderer.invoke('showTracklistMenu', {
			id: 'root',
			isFolder: false,
			isRoot: true,
		})
	}

	let root_droppable = $state(false)
	function dragover(e: DragEvent) {
		if (e.currentTarget && e.dataTransfer?.types[0] === 'ferrum.playlist') {
			root_droppable = true
			e.preventDefault()
		}
	}
	function dragleave() {
		root_droppable = false
	}
	function drop(e: DragEvent) {
		if (e.currentTarget && e.dataTransfer?.types[0] === 'ferrum.playlist' && dragged.playlist) {
			const root = $track_lists_details_map['root']
			if (!root.children) {
				return
			}
			move_playlist(dragged.playlist.id, dragged.playlist.from_folder, 'root', root.children.length)
			root_droppable = false
		}
	}

	let content_element: HTMLDivElement | undefined = $state()

	async function scroll_to_active() {
		await tick()
		const active = content_element?.querySelector('.active')
		if (content_element && active instanceof HTMLElement) {
			const top = active.offsetTop
			if (content_element.scrollTop > top) {
				content_element.scrollTop = top
			} else if (
				content_element.scrollTop + content_element.clientHeight <
				top + active.clientHeight
			) {
				content_element.scrollTop = top + active.clientHeight - content_element.clientHeight
			}
		}
	}
	onDestroy(
		current_playlist_id.subscribe(() => {
			scroll_to_active()
		}),
	)

	/** Prevent focus weirdness */
	function focuser() {
		if (content_element) {
			const scroll_top = content_element.scrollTop
			viewport?.focus()
			content_element.scrollTop = scroll_top
			scroll_to_active()
		}
	}
</script>

<!-- NOTE: aside is used as css selector in SidebarItems -->
<aside onmousedown={prevent_default_self} role="none">
	{#if is_mac}
		<div class="titlebar" onmousedown={prevent_default_self} role="none"></div>
	{/if}
	<div class="content" bind:this={content_element}>
		<Filter
			onfocus={() => {
				if (content_element) {
					content_element.scrollTop = 0
				}
			}}
			onkeydown={(e) => {
				if (e.key === 'Escape') {
					tracklist_actions.focus()
				}
			}}
		/>
		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
		<nav
			class="items"
			tabindex="-1"
			onmousedown={(e) => {
				e.preventDefault()
				if (document.activeElement === document.body) {
					tracklist_actions.focus()
				}
			}}
			onkeydown={(e) => {
				if (e.key === 'Escape') {
					e.preventDefault()
					tracklist_actions.focus()
				} else if (
					e.key === 'Home' ||
					e.key === 'End' ||
					e.key === 'PageUp' ||
					e.key === 'PageDown'
				) {
					e.preventDefault()
				} else {
					$item_handle?.handleKey(e)
				}
			}}
			bind:this={viewport}
			class:droppable={root_droppable}
			oncontextmenu={(e) => {
				if (e.target === e.currentTarget) {
					on_context_menu()
				}
			}}
			ondragover={(e) => {
				if (e.target === e.currentTarget) {
					dragover(e)
				}
			}}
			ondragleave={(e) => {
				if (e.target === e.currentTarget) {
					dragleave()
				}
			}}
			ondrop={(e) => {
				if (e.target === e.currentTarget) {
					drop(e)
				}
			}}
		>
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<div class="focuser" tabindex="0" onfocus={focuser}></div>
			<div class="spacer"></div>
			<SidebarItems
				parent_id={null}
				children={special_playlists_nav}
				on_select_down={() => {
					if ($track_lists_details_map.root.children && $track_lists_details_map.root.children[0]) {
						navigate('/playlist/' + $track_lists_details_map.root.children[0])
					}
				}}
			/>
			<div class="spacer"></div>
			<SidebarItems
				parent_id="root"
				children={($track_lists_details_map['root'].children || []).map(
					(child_id) => $track_lists_details_map[child_id],
				)}
			/>
		</nav>
	</div>
</aside>

<style lang="sass">
	aside
		width: 230px
		min-width: 230px
		display: flex
		flex-direction: column
		background-color: hsla(0, 0%, 0%, 0.7)
		box-shadow: inset -6px 0px 6px -6px #000000
	.titlebar
		-webkit-app-region: drag
		height: var(--titlebar-height)
		flex-shrink: 0
	.content
		padding-top: 5px
		overflow-y: auto
		display: flex
		flex-direction: column
		flex-grow: 1
		background-color: rgba(0, 0, 0, 0.01) // for scrollbar color
		position: relative // for SidebarItems scrollToIndex
	.spacer
		height: 10px
		flex-shrink: 0
		pointer-events: none
	.items
		width: 100%
		flex-grow: 1
		font-size: 13px
		outline: none
		background-color: inherit
		display: flex
		flex-direction: column
		&:focus .focuser
			display: none
	.droppable
		box-shadow: inset 0px 0px 0px 2px var(--accent-1)
		background-color: hsla(var(--hue), 74%, 53%, 0.1)
</style>
