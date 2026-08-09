<script lang="ts">
	import { run } from 'svelte/legacy'

	import { onDestroy } from 'svelte'
	import { check_shortcut } from '../lib/helpers'
	import { ipc_listen } from '$lib/window'
	import fuzzysort from 'fuzzysort'
	import { track_lists_details_map } from '$lib/data'
	import type { TrackListDetails } from '../../ferrum-addon/addon'
	import Modal from './Modal.svelte'
	import { special_playlists_nav } from './Sidebar.svelte'
	import { navigate } from '$lib/router'

	type Result = TrackListDetails & { path?: string }

	let value = $state('')
	let playlists: Result[] = $state([])
	let show = $state(false)
	function get_playlists() {
		const playlists: Result[] = [...special_playlists_nav]
		for (const playlist of Object.values($track_lists_details_map)) {
			if (playlist.kind === 'playlist' || playlist.kind === 'folder') {
				playlists.push(playlist)
			}
		}
		return playlists
	}

	let filtered_items = $state(fuzzysort.go(value, playlists, { key: 'name', all: true }))

	function select_input(el: HTMLInputElement) {
		el.select()
	}

	let selected_index = $state(0)

	function clamp_index() {
		selected_index = Math.max(0, Math.min(filtered_items.length - 1, selected_index))
	}
	let list_items: HTMLElement[] = $state([])

	function handle_keydown(e: KeyboardEvent) {
		if (e.key === 'Tab') {
			e.preventDefault()
		} else if (check_shortcut(e, 'Escape')) {
			show = false
			value = ''
		} else if (check_shortcut(e, 'Enter')) {
			navigate('/playlist/' + filtered_items[selected_index].obj.id)
			show = false
		} else if (check_shortcut(e, 'ArrowUp')) {
			selected_index--
			clamp_index()
			list_items[selected_index].scrollIntoView({
				block: 'nearest',
			})
			e.preventDefault()
		} else if (check_shortcut(e, 'ArrowDown')) {
			selected_index++
			clamp_index()
			list_items[selected_index].scrollIntoView({
				block: 'nearest',
			})
			e.preventDefault()
		}
	}

	onDestroy(
		ipc_listen('ToggleQuickNav', () => {
			show = !show
		}),
	)
	run(() => {
		if (show) {
			playlists = get_playlists()
		}
	})
	run(() => {
		filtered_items = fuzzysort.go(value, playlists, { key: 'name', all: true })
		clamp_index()
	})
	run(() => {
		;(list_items, clamp_index())
	})
</script>

{#if show}
	<Modal
		plain
		on_cancel={() => {
			show = false
		}}
	>
		<input
			type="text"
			bind:value
			onkeydown={handle_keydown}
			placeholder="Search for a playlist..."
			use:select_input
		/>
		<div class="items-container">
			{#each filtered_items as item, i}
				<button
					bind:this={list_items[i]}
					type="button"
					onclick={() => {
						navigate(item.obj.path ?? '/playlist/' + item.obj.id)
						show = false
					}}
					class:selected={selected_index === i}
				>
					{#if item.obj.kind === 'folder'}
						<svg
							style="padding: 1px"
							xmlns="http://www.w3.org/2000/svg"
							height="22px"
							viewBox="0 0 24 24"
							width="22px"
							fill="currentColor"
							><path d="M0 0h24v24H0V0z" fill="none" /><path
								d="M9.17 6l2 2H20v10H4V6h5.17M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"
							/></svg
						>
					{:else}
						<svg
							style="transform: translateX(2px)"
							xmlns="http://www.w3.org/2000/svg"
							height="24px"
							viewBox="0 0 24 24"
							width="24px"
							fill="currentColor"
							><path
								d="M5 10h10c.55 0 1 .45 1 1s-.45 1-1 1H5c-.55 0-1-.45-1-1s.45-1 1-1zm0-4h10c.55 0 1 .45 1 1s-.45 1-1 1H5c-.55 0-1-.45-1-1s.45-1 1-1zm0 8h6c.55 0 1 .45 1 1s-.45 1-1 1H5c-.55 0-1-.45-1-1s.45-1 1-1zm9 .88v4.23c0 .39.42.63.76.43l3.53-2.12c.32-.19.32-.66 0-.86l-3.53-2.12c-.34-.19-.76.05-.76.44z"
							/></svg
						>
					{/if}
					<span>{item.obj.name}</span>
				</button>
			{/each}
		</div>
	</Modal>
{/if}

<style lang="sass">
	.items-container
		width: 90vw
		max-width: 580px
		height: 305px
		max-height: 100%
	input
		font-size: inherit
		font-family: inherit
		padding: 16px 20px
		border-radius: 7px
		width: 100%
		box-sizing: border-box
		outline: none
		border: none
		background-color: transparent
		color: inherit
	.items-container
		border-top: 1px solid rgba(#ffffff, 0.08)
		padding: 8px
		scroll-padding: 8px
		overflow-y: auto
	button
		color: inherit
		font-size: inherit
		font-family: inherit
		background-color: inherit
		border: none
		display: flex
		align-items: center
		gap: 10px
		border-radius: 7px
		width: 100%
		padding: 10px
		&:hover
			background-color: rgba(#ffffff, 0.05)
		&.selected
			background-color: rgba(#ffffff, 0.15)
</style>
