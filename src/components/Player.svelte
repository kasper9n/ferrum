<script lang="ts">
	import {
		stopped,
		play_pause,
		seek,
		playing_track,
		playing_id,
		previous,
		skip_to_next,
		cover_src,
		volume,
		time_record,
	} from '../lib/player'
	import { get_duration, prevent_default } from '../lib/helpers'
	import { queue_visible, queue, shuffle, repeat } from '../lib/queue'
	import { lyrics_state, toggle_lyrics_visibility } from '$lib/lyrics.svelte'
	import { get_track, is_dev } from '$lib/data'
	import { dragged } from '$lib/drag-drop'
	import * as dragGhost from './DragGhost.svelte'
	import Slider from './Slider.svelte'
	import {
		get_show_in_playlists_tree,
		get_tracklists_tree,
		handle_selected_tracks_action,
	} from '$lib/menus'
	import { ipc_renderer } from '$lib/window'
	import { tracks_page_item_ids } from './TrackList.svelte'
	import { onDestroy } from 'svelte'

	interface Props {
		on_toggle_visualizer: () => void
	}
	let { on_toggle_visualizer }: Props = $props()

	function toggle_queue() {
		$queue_visible = !$queue_visible
		lyrics_state.visible = false
	}
	$effect(() => {
		ipc_renderer.invoke('update:Show Queue', $queue_visible)
	})
	ipc_renderer.on('Show Queue', toggle_queue)
	onDestroy(() => {
		ipc_renderer.removeListener('Show Queue', toggle_queue)
	})

	function toggle_lyrics() {
		toggle_lyrics_visibility()
	}
	ipc_renderer.on('Show Lyrics', toggle_lyrics)
	onDestroy(() => {
		ipc_renderer.removeListener('Show Lyrics', toggle_lyrics)
	})

	async function playing_context_menu() {
		const playing = queue.getCurrent()
		if (playing) {
			const action = await ipc_renderer.invoke('show_tracks_menu', {
				is_editable_playlist: false,
				queue: false,
				lists: get_tracklists_tree(),
				show_in_playlists: get_show_in_playlists_tree([playing.id]),
			})
			if (action !== null) {
				handle_selected_tracks_action({
					action,
					track_ids: [playing.id],
					all_ids: [playing.id],
					first_index: 0,
				})
			}
		}
	}

	function drag_start(e: DragEvent) {
		if (e.dataTransfer && $playing_id) {
			e.dataTransfer.effectAllowed = 'move'
			const track = get_track($playing_id)
			dragGhost.set_inner_text(track.artist + ' - ' + track.name)
			dragged.tracks = {
				ids: [$playing_id],
			}
			e.dataTransfer.setDragImage(dragGhost.drag_el, 0, 0)
			e.dataTransfer.setData('ferrum.tracks', '')
		}
	}
</script>

<div class="player" class:stopped={$stopped} class:dev={is_dev}>
	<div class="left">
		{#if !$stopped && $playing_track}
			<div
				class="cover"
				role="none"
				oncontextmenu={playing_context_menu}
				ondragstart={drag_start}
				draggable="true"
			>
				{#if $cover_src}
					<img src={$cover_src} alt="" draggable="false" />
				{:else}
					<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
						<path
							d="M23 0l-15.996 3.585v13.04c-2.979-.589-6.004 1.671-6.004 4.154 0 2.137 1.671 3.221 3.485 3.221 2.155 0 4.512-1.528 4.515-4.638v-10.9l12-2.459v8.624c-2.975-.587-6 1.664-6 4.141 0 2.143 1.715 3.232 3.521 3.232 2.14 0 4.476-1.526 4.479-4.636v-17.364z"
						/>
					</svg>
				{/if}
			</div>
			<div
				oncontextmenu={playing_context_menu}
				ondragstart={drag_start}
				draggable="true"
				role="none"
			>
				<div class="name">
					{$playing_track.name}
				</div>
				<div class="artist">
					{$playing_track.artist}
				</div>
			</div>
		{/if}
	</div>
	<div class="middle">
		<div class="controls">
			<button
				type="button"
				aria-label="Shuffle"
				class="side-controls shuffle"
				class:on={$shuffle}
				tabindex="-1"
				onmousedown={prevent_default}
				onclick={() => ($shuffle = !$shuffle)}
			>
				<div class="parent-active-zoom">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="scale-y-[0.85]"
						width="24"
						viewBox="-8 -8 40 40"
						height="24"
						><path
							d="M2 7h-2v-2h2c3.49 0 5.48 1.221 6.822 2.854-.41.654-.754 1.312-1.055 1.939-1.087-1.643-2.633-2.793-5.767-2.793zm16 10c-3.084 0-4.604-1.147-5.679-2.786-.302.627-.647 1.284-1.06 1.937 1.327 1.629 3.291 2.849 6.739 2.849v3l6-4-6-4v3zm0-10v3l6-4-6-4v3c-5.834 0-7.436 3.482-8.85 6.556-1.343 2.921-2.504 5.444-7.15 5.444h-2v2h2c5.928 0 7.543-3.511 8.968-6.609 1.331-2.893 2.479-5.391 7.032-5.391z"
						/></svg
					>
				</div>
			</button>
			<button
				type="button"
				aria-label="Previous"
				class="previous"
				onclick={previous}
				tabindex="-1"
				onmousedown={prevent_default}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="parent-active-zoom"
					height="24"
					viewBox="0 0 24 24"
					width="24"
				>
					<path
						d="M7 6c.55 0 1 .45 1 1v10c0 .55-.45 1-1 1s-1-.45-1-1V7c0-.55.45-1 1-1zm3.66 6.82l5.77 4.07c.66.47 1.58-.01 1.58-.82V7.93c0-.81-.91-1.28-1.58-.82l-5.77 4.07c-.57.4-.57 1.24 0 1.64z"
					/>
				</svg>
			</button>

			<button
				type="button"
				class="play-pause"
				onclick={play_pause}
				class:cannot-play={$tracks_page_item_ids.length === 0}
				tabindex="-1"
				onmousedown={prevent_default}
			>
				{#if $time_record.paused}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="parent-active-zoom"
						height="36"
						viewBox="0 0 24 24"
						width="36"
					>
						<path
							d="M8 6.82v10.36c0 .79.87 1.27 1.54.84l8.14-5.18c.62-.39.62-1.29 0-1.69L9.54 5.98C8.87 5.55 8 6.03 8 6.82z"
						/>
					</svg>
				{:else}
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="parent-active-zoom"
						height="36"
						viewBox="0 0 24 24"
						width="36"
					>
						<path
							d="M8 19c1.1 0 2-.9 2-2V7c0-1.1-.9-2-2-2s-2 .9-2 2v10c0 1.1.9 2 2 2zm6-12v10c0 1.1.9 2 2 2s2-.9 2-2V7c0-1.1-.9-2-2-2s-2 .9-2 2z"
						/>
					</svg>
				{/if}
			</button>

			<button
				type="button"
				aria-label="Next"
				class="next"
				onclick={skip_to_next}
				tabindex="-1"
				onmousedown={prevent_default}
			>
				<svg
					xmlns="http://www.w3.org/2000/svg"
					class="parent-active-zoom"
					height="24"
					viewBox="0 0 24 24"
					width="24"
				>
					<path
						d="M7.58 16.89l5.77-4.07c.56-.4.56-1.24 0-1.63L7.58 7.11C6.91 6.65 6 7.12 6 7.93v8.14c0 .81.91 1.28 1.58.82zM16 7v10c0 .55.45 1 1 1s1-.45 1-1V7c0-.55-.45-1-1-1s-1 .45-1 1z"
					/>
				</svg>
			</button>

			<button
				type="button"
				aria-label="Repeat"
				class="side-controls repeat"
				class:on={$repeat}
				tabindex="-1"
				onmousedown={prevent_default}
				onclick={() => ($repeat = !$repeat)}
			>
				<div class="parent-active-zoom">
					<svg
						xmlns="http://www.w3.org/2000/svg"
						class="scale-y-105"
						width="24"
						height="24"
						viewBox="-8 -8 40 40"
						><path
							d="M2 12c0 .999.381 1.902.989 2.604l-1.098.732-.587.392c-.814-1.025-1.304-2.318-1.304-3.728 0-3.313 2.687-6 6-6h9v-3l6 4-6 4v-3h-9c-2.206 0-4 1.794-4 4zm20.696-3.728l-.587.392-1.098.732c.608.702.989 1.605.989 2.604 0 2.206-1.795 4-4 4h-9v-3l-6 4 6 4v-3h9c3.313 0 6-2.687 6-6 0-1.41-.489-2.703-1.304-3.728z"
						/></svg
					>
				</div>
			</button>
		</div>
		<div class="time-bar">
			<small class="current-time">{get_duration($time_record.elapsed)}</small>
			<Slider
				class="mx-1.5 w-full"
				value={$time_record.elapsed / $time_record.duration || 0}
				growth_rate={$time_record.paused ? 0 : 1 / $time_record.duration}
				max={1}
				update_on_drag={false}
				on_user_change={(value) => {
					seek(value * $time_record.duration)
				}}
			/>
			<small class="duration">{get_duration($time_record.duration)}</small>
		</div>
	</div>
	<div class="right">
		<button
			type="button"
			class="mr-2"
			aria-label="Toggle visualizer"
			tabindex="-1"
			onclick={() => {
				on_toggle_visualizer()
			}}
		>
			<div class="parent-active-zoom">
				<svg
					xmlns="http://www.w3.org/2000/svg"
					width="24"
					height="24"
					viewBox="-2 -2 26 26"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					class="lucide lucide-audio-lines-icon lucide-audio-lines"
					><path d="M2 10v3" /><path d="M6 6v11" /><path d="M10 3v18" /><path d="M14 8v7" /><path
						d="M18 5v13"
					/><path d="M22 10v3" /></svg
				>
			</div>
		</button>
		<button type="button" class="volume-icon" tabindex="-1" onclick={volume.toggle}>
			{#if $volume > 0.5}
				<svg
					class="high parent-active-zoom"
					xmlns="http://www.w3.org/2000/svg"
					height="24px"
					viewBox="0 0 24 24"
					width="24px"
					fill="#000000"
				>
					<path
						d="M3 10v4c0 .55.45 1 1 1h3l3.29 3.29c.63.63 1.71.18 1.71-.71V6.41c0-.89-1.08-1.34-1.71-.71L7 9H4c-.55 0-1 .45-1 1zm13.5 2c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 4.45v.2c0 .38.25.71.6.85C17.18 6.53 19 9.06 19 12s-1.82 5.47-4.4 6.5c-.36.14-.6.47-.6.85v.2c0 .63.63 1.07 1.21.85C18.6 19.11 21 15.84 21 12s-2.4-7.11-5.79-8.4c-.58-.23-1.21.22-1.21.85z"
					/>
				</svg>
			{:else if $volume > 0}
				<svg
					class="low parent-active-zoom"
					xmlns="http://www.w3.org/2000/svg"
					height="24px"
					viewBox="0 0 24 24"
					width="24px"
					fill="#000000"
				>
					<path
						d="M18.5 12c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM5 10v4c0 .55.45 1 1 1h3l3.29 3.29c.63.63 1.71.18 1.71-.71V6.41c0-.89-1.08-1.34-1.71-.71L9 9H6c-.55 0-1 .45-1 1z"
					/>
				</svg>
			{:else}
				<svg
					class="mute parent-active-zoom"
					xmlns="http://www.w3.org/2000/svg"
					height="24px"
					viewBox="0 0 24 24"
					width="24px"
					fill="#000000"
				>
					<path
						d="M7 10v4c0 .55.45 1 1 1h3l3.29 3.29c.63.63 1.71.18 1.71-.71V6.41c0-.89-1.08-1.34-1.71-.71L11 9H8c-.55 0-1 .45-1 1z"
					/>
				</svg>
			{/if}
		</button>
		<Slider class="mr-2 w-[110px]" bind:value={$volume} max={1} />
		<button
			type="button"
			class="mr-2"
			aria-label="Toggle lyrics"
			tabindex="-1"
			onmousedown={(e) => e.preventDefault()}
			onclick={toggle_lyrics}
			class:on={lyrics_state.visible}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="parent-active-zoom lyrics-icon"
				height="24px"
				viewBox="0 0 24 24"
				width="24px"
				style="fill:none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path stroke="none" d="M0 0h24v24H0z" fill="none" />
				<path d="M15 12.9a5 5 0 1 0 -3.902 -3.9" />
				<path d="M15 12.9l-3.902 -3.899l-7.513 8.584a2 2 0 1 0 2.827 2.83l8.588 -7.515" />
			</svg>
		</button>
		<button
			type="button"
			aria-label="Toggle queue"
			tabindex="-1"
			onmousedown={(e) => e.preventDefault()}
			onclick={toggle_queue}
			class:on={$queue_visible}
		>
			<svg
				xmlns="http://www.w3.org/2000/svg"
				class="parent-active-zoom"
				height="24px"
				viewBox="0 0 24 24"
				width="24px"
			>
				<path
					d="M5 10h10c.55 0 1 .45 1 1s-.45 1-1 1H5c-.55 0-1-.45-1-1s.45-1 1-1zm0-4h10c.55 0 1 .45 1 1s-.45 1-1 1H5c-.55 0-1-.45-1-1s.45-1 1-1zm0 8h6c.55 0 1 .45 1 1s-.45 1-1 1H5c-.55 0-1-.45-1-1s.45-1 1-1zm9 .88v4.23c0 .39.42.63.76.43l3.53-2.12c.32-.19.32-.66 0-.86l-3.53-2.12c-.34-.19-.76.05-.76.44z"
				/>
			</svg>
		</button>
	</div>
</div>

<style lang="sass">
	svg
		fill: var(--icon-color)
		display: block
	.player
		display: flex
		height: 80px
		padding: 8px
		box-sizing: border-box
		align-items: center
		border-top: 1px solid var(--border-color)
		background-color: var(--player-bg-color)
		&:not(:hover).dev
			background: linear-gradient(90deg, hsl(205deg 60% 15%), hsl(280deg 60% 15%))
	.stopped
		.time-bar, .side-controls, .previous, .next, .play-pause.cannot-play
			pointer-events: none
			opacity: 0.35
	.left
		width: 30%
		display: flex
		align-items: center
		height: 100%
		.cover
			height: 100%
		img
			height: 100%
			max-width: 120px
			object-fit: contain
		svg
			height: 100%
			padding: 18px
			width: auto
			box-sizing: border-box
			background: var(--empty-cover-bg-color)
			fill: var(--empty-cover-color)
			border-radius: 2px
		.name
			margin-left: 15px
			font-size: 14px
		.artist
			margin-left: 15px
			font-size: 12px
			opacity: 0.8
	.middle
		width: 40%
	.right
		width: 30%
		display: flex
		justify-content: flex-end
		align-items: center
		padding-right: 8px
		box-sizing: border-box
	button
		background-color: transparent
		outline: none
		border: none
		padding: 0px
	button.side-controls
		margin: 0px 18px
		svg
			opacity: 0.8
		&.on svg
			opacity: 0.9
	button.play-pause
		margin: 0px 14px
	.controls
		display: flex
		justify-content: center
		align-items: center
	.time-bar
		display: flex
		align-items: center
		margin-bottom: 2px
		.current-time
			text-align: right
		small
			font-size: 13px
			opacity: 0.5
			min-width: 40px
			font-family: 'Open Sans' // for monospace digits
	.on svg
		fill: var(--icon-highlight-color)
	.on .lyrics-icon
		stroke: var(--icon-highlight-color)
	button.volume-icon
		width: 24px
		padding-right: 4px
		box-sizing: content-box
		.high
			translate: 4px
		.low
			translate: 2px
	.lyrics-icon
		width: 18px
		height: 18px
		stroke-width: 2.4
</style>
