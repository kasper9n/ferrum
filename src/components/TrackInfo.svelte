<script lang="ts" module>
	export type TrackInfoInstance = {
		ids: TrackID[]
		index: number
		id: TrackID
		unedited_info: ReturnType<typeof get_info>
		info: ReturnType<typeof get_info>
		image_index: number
	}

	export const track_info_state = $state({
		instance: null as TrackInfoInstance | null,
	})

	export function open_track_info(ids: TrackID[], index: number) {
		if (get(modal_count) === 0 || track_info_state.instance !== null) {
			const id = ids[index]
			const track = get_track(ids[index])
			const result = load_tags(id)
			if (result.error) {
				cancel()
			} else {
				track_info_state.instance = {
					ids,
					index,
					id,
					unedited_info: get_info(track),
					info: get_info(track),
					image_index: 0,
				}
			}
		}
	}

	function cancel() {
		track_info_state.instance = null
	}

	function to_string(value: unknown) {
		return String(value).replace(/\0/g, '') // remove NULL bytes
	}

	function get_info(track: Track) {
		return {
			image_edited: false,
			name: track.name,
			artist: track.artist,
			album_name: track.albumName || '',
			album_artist: track.albumArtist || '',
			composer: track.composer || '',
			grouping: track.grouping || '',
			genre: track.genre || '',
			year: to_string(track.year || ''),
			track_num: to_string(track.trackNum || ''),
			track_count: to_string(track.trackCount || ''),
			disc_num: to_string(track.discNum || ''),
			disc_count: to_string(track.discCount || ''),
			bpm: to_string(track.bpm || ''),
			compilation: track.compilation || false,
			rating: track.rating || 0,
			liked: track.liked || false,
			play_count: track.playCount || 0,
			comments: to_string(track.comments || ''),
		}
	}

	ipc_listen('context.Get Info', (_, ids: TrackID[], track_index: number) => {
		open_track_info(ids, track_index)
	})
</script>

<script lang="ts">
	import { check_shortcut } from '$lib/helpers'
	import Button from './Button.svelte'
	import type { Track, TrackID } from '../../ferrum-addon'
	import { ipc_listen, ipc_renderer } from '$lib/window'
	import { playing_id, reload } from '$lib/player'
	import { get } from 'svelte/store'
	import Modal, { modal_count } from './Modal.svelte'
	import {
		get_genres,
		get_image,
		get_track,
		load_tags,
		remove_image,
		set_image,
		set_image_data,
		update_track_info,
	} from '$lib/data'
	import { tick } from 'svelte'

	const genres = get_genres()

	let { instance = $bindable() }: { instance: TrackInfoInstance } = $props()

	function uint_filter(value: string) {
		return value.replace(/[^0-9]*/g, '')
	}

	let info = $derived(instance.info)
	$effect(() => {
		info.year = uint_filter(info.year)
	})

	function is_edited() {
		return JSON.stringify(instance.info) !== JSON.stringify(instance.unedited_info)
	}
	function save(hide_after = true) {
		if (is_edited()) {
			update_track_info(instance.id, {
				name: info.name,
				artist: info.artist,
				albumName: info.album_name,
				albumArtist: info.album_artist,
				composer: info.composer,
				grouping: info.grouping,
				genre: info.genre,
				year: info.year,
				trackNum: info.track_num,
				trackCount: info.track_count,
				discNum: info.disc_num,
				discCount: info.disc_count,
				bpm: info.bpm,
				// compilation,
				// rating,
				// liked,
				// playCount,
				comments: info.comments,
			})
			if (instance.id === $playing_id) {
				reload()
			}
		}
		if (hide_after) {
			cancel()
		}
	}

	function get_track_info_image(index: number) {
		const result = get_image(index)
		if (result.error) {
			return null
		}
		return result.data
	}

	let image = $derived(get_track_info_image(instance.image_index))
	/** Undefined when loading, null when no image exists */
	let object_url = $state<string | null | undefined>()

	// First, we set the object_url to undefined (loading)
	$effect(() => {
		const new_image = image
		object_url = undefined

		// We set the object URL asynchronously, so that the loading state is shown
		if (new_image === null) {
			object_url = null
			return
		}
		const object_url_promise = tick().then(() => {
			object_url = URL.createObjectURL(new Blob([new Uint8Array(new_image.data)]))
			return object_url
		})

		return async () => {
			const object_url_to_revoke = await object_url_promise
			if (object_url_to_revoke) {
				URL.revokeObjectURL(object_url_to_revoke)
			}
		}
	})

	function prev_image() {
		if (image && image.index >= 1) {
			instance.image_index = image.index - 1
		}
	}
	function next_image() {
		if (image && image.index < image.totalImages - 1) {
			instance.image_index = image.index + 1
		}
	}

	function big(v: string) {
		return v.length >= 3
	}
	function keydown(e: KeyboardEvent) {
		if (check_shortcut(e, '[', { cmd_or_ctrl: true })) {
			save(false)
			if (instance && instance.index - 1 >= 0) {
				open_track_info(instance.ids, instance.index - 1)
			}
			e.preventDefault()
		} else if (check_shortcut(e, ']', { cmd_or_ctrl: true })) {
			save(false)
			if (instance && instance.index + 1 <= instance.ids.length - 1) {
				open_track_info(instance.ids, instance.index + 1)
			}

			e.preventDefault()
		}
	}
	function keydown_none_selected(e: KeyboardEvent) {
		if (check_shortcut(e, 'Enter')) {
			save()
		}
	}

	let droppable = $state(false)
	const allowed_mimes = ['image/jpeg', 'image/png']
	function get_file_path(e: DragEvent): File | null {
		if (e.dataTransfer && has_file(e)) {
			for (let i = 0; i < e.dataTransfer.files.length; i++) {
				const file = e.dataTransfer.files[i]
				if (allowed_mimes.includes(file.type)) {
					return file.path
				}
			}
		}
		return null
	}
	function has_file(e: DragEvent): boolean {
		if (!e.dataTransfer) return false
		let count = 0
		for (let i = 0; i < e.dataTransfer.items.length; i++) {
			const item = e.dataTransfer.items[i]
			if (item.kind === 'file' && allowed_mimes.includes(item.type)) {
				count++
			}
		}
		return count === 1
	}
	function drag_enter_or_over(e: DragEvent) {
		e.preventDefault()
		droppable = has_file(e)
	}
	function drag_leave(e: DragEvent) {
		e.preventDefault()
		droppable = false
	}
	function drop(e: DragEvent) {
		e.preventDefault()
		droppable = false
		const path = get_file_path(e)
		if (path !== null) {
			replace_cover(path)
		}
	}
	function cover_keydown(e: KeyboardEvent) {
		if (check_shortcut(e, 'Backspace') && image) {
			const result = remove_image(image.index)
			if (result.error) {
				cancel()
				return
			}
			info.image_edited = true
			if (image.index < image.totalImages - 1) {
				instance.image_index = image.index
			} else {
				instance.image_index = Math.max(image.index - 1)
			}
		} else if (check_shortcut(e, ' ')) {
			pick_cover()
		} else if (check_shortcut(e, 'ArrowLeft')) {
			prev_image()
		} else if (check_shortcut(e, 'ArrowRight')) {
			next_image()
		} else {
			keydown_none_selected(e)
		}
	}
	async function pick_cover() {
		let result = await ipc_renderer.invoke('showOpenDialog', false, {
			properties: ['openFile'],
			filters: [{ name: 'Images', extensions: ['jpg', 'jpeg', 'png'] }],
		})
		if (!result.canceled && result.filePaths.length === 1) {
			replace_cover(result.filePaths[0])
		}
	}
	function replace_cover(file_path: string) {
		const result = set_image(image?.index ?? 0, file_path)
		if (!result.error) {
			info.image_edited = true
			instance.image_index = image?.index ?? 0
			image = get_track_info_image(instance.image_index)
		}
	}
	function replace_cover_data(data: ArrayBuffer) {
		const result = set_image_data(image?.index ?? 0, data)
		if (!result.error) {
			info.image_edited = true
			instance.image_index = image?.index ?? 0
			image = get_track_info_image(instance.image_index)
		}
	}
	function cover_paste(e: ClipboardEvent) {
		if (e.clipboardData && e.clipboardData.files.length === 1) {
			const file = e.clipboardData.files[0]
			if (allowed_mimes.includes(file.type) && file.path !== '' && file.path) {
				replace_cover(file.path)
			} else if (allowed_mimes.includes(file.type)) {
				const reader = new FileReader()
				reader.onload = (e) => {
					if (e.target?.result && e.target.result instanceof ArrayBuffer) {
						replace_cover_data(e.target.result)
					}
				}
				reader.readAsArrayBuffer(file)
			}
		}
	}
</script>

<svelte:window onkeydown={keydown} />
<svelte:body
	onkeydown={(e) => {
		if (e.target === e.currentTarget) {
			keydown_none_selected(e)
		}
	}}
	onpaste={cover_paste}
/>
<Modal on_cancel={cancel} cancel_on_escape form={save}>
	<main class="modal">
		<div class="header" class:has-subtitle={image && image.totalImages >= 2}>
			<div
				class="cover-area"
				class:droppable
				tabindex="0"
				onkeydown={cover_keydown}
				role="button"
				aria-label="Cover artwork"
			>
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="cover"
					ondragenter={drag_enter_or_over}
					ondragover={drag_enter_or_over}
					ondragleave={drag_leave}
					ondrop={drop}
					ondblclick={pick_cover}
				>
					{#if object_url}
						<img class="outline-element" alt="" src={object_url} />
					{:else if object_url === null}
						<svg
							class="cover-svg outline-element"
							xmlns="http://www.w3.org/2000/svg"
							width="8"
							height="8"
							viewBox="0 0 24 24"
						>
							<path
								d="M23 0l-15.996 3.585v13.04c-2.979-.589-6.004 1.671-6.004 4.154 0 2.137 1.671 3.221 3.485 3.221 2.155 0 4.512-1.528 4.515-4.638v-10.9l12-2.459v8.624c-2.975-.587-6 1.664-6 4.141 0 2.143 1.715 3.232 3.521 3.232 2.14 0 4.476-1.526 4.479-4.636v-17.364z"
							/>
						</svg>
					{:else}
						<!-- Show nothing when loading -->
					{/if}
				</div>
				{#if image && image.totalImages >= 2}
					{@const image_index = image.index}
					<div class="cover-subtitle">
						<div class="arrow" class:unclickable={image_index <= 0}>
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<svg
								onclick={prev_image}
								tabindex="-1"
								role="button"
								aria-label="Previous image"
								clip-rule="evenodd"
								fill-rule="evenodd"
								stroke-linejoin="round"
								stroke-miterlimit="2"
								width="18"
								height="18"
								fill="currentColor"
								viewBox="0 0 24 24"
								xmlns="http://www.w3.org/2000/svg"
								><path
									d="m13.789 7.155c.141-.108.3-.157.456-.157.389 0 .755.306.755.749v8.501c0 .445-.367.75-.755.75-.157 0-.316-.05-.457-.159-1.554-1.203-4.199-3.252-5.498-4.258-.184-.142-.29-.36-.29-.592 0-.23.107-.449.291-.591 1.299-1.002 3.945-3.044 5.498-4.243z"
								/></svg
							>
						</div>
						<div class="subtitle-text">
							{image.index + 1} / {image.totalImages}
						</div>
						<div class="arrow" class:unclickable={image_index >= image.totalImages - 1}>
							<!-- svelte-ignore a11y_click_events_have_key_events -->
							<svg
								onclick={next_image}
								tabindex="-1"
								role="button"
								aria-label="Next image"
								clip-rule="evenodd"
								fill-rule="evenodd"
								stroke-linejoin="round"
								stroke-miterlimit="2"
								width="18"
								height="18"
								fill="currentColor"
								viewBox="0 0 24 24"
								xmlns="http://www.w3.org/2000/svg"
								><path
									d="m10.211 7.155c-.141-.108-.3-.157-.456-.157-.389 0-.755.306-.755.749v8.501c0 .445.367.75.755.75.157 0 .316-.05.457-.159 1.554-1.203 4.199-3.252 5.498-4.258.184-.142.29-.36.29-.592 0-.23-.107-.449-.291-.591-1.299-1.002-3.945-3.044-5.498-4.243z"
								/></svg
							>
						</div>
					</div>
				{/if}
			</div>
			<div class="text">
				<div class="name">{info.name}</div>
				<div class="artist">{info.artist}</div>
			</div>
		</div>
		<div class="spacer"></div>
		<div class="row">
			<div class="label">Title</div>
			<!-- svelte-ignore a11y_autofocus -->
			<input type="text" bind:value={info.name} autofocus />
		</div>
		<div class="row">
			<div class="label">Artist</div>
			<input type="text" bind:value={info.artist} />
		</div>
		<div class="row">
			<div class="label">Album</div>
			<input type="text" bind:value={info.album_name} />
		</div>
		<div class="row">
			<div class="label">Album artist</div>
			<input type="text" bind:value={info.album_artist} />
		</div>
		<div class="row">
			<div class="label">Composer</div>
			<input type="text" bind:value={info.composer} />
		</div>
		<div class="row">
			<div class="label">Grouping</div>
			<input type="text" bind:value={info.grouping} />
		</div>
		<div class="row">
			<div class="label">Genre</div>
			<input
				type="text"
				bind:value={info.genre}
				oninput={(e) => {
					if (!(e instanceof InputEvent) || e.inputType !== 'insertText') {
						return
					}
					const match = genres.find((genre) => {
						return genre.startsWith(e.currentTarget.value)
					})
					if (!match) {
						return
					}
					const start = e.currentTarget.selectionStart
					info.genre = match
					e.currentTarget.value = info.genre
					e.currentTarget.setSelectionRange(start, info.genre.length) // select the appended text
				}}
			/>
		</div>
		<div class="row">
			<div class="label">Year</div>
			<input class="medium" type="text" bind:value={info.year} />
		</div>
		<div class="row num">
			<div class="label">Track</div>
			<input class="num" type="text" bind:value={info.track_num} class:big={big(info.track_num)} />
			<div class="midtext">of</div>
			<input
				class="num"
				type="text"
				bind:value={info.track_count}
				class:big={big(info.track_count)}
			/>
		</div>
		<div class="row num">
			<div class="label">Disc number</div>
			<input class="num" type="text" bind:value={info.disc_num} class:big={big(info.disc_num)} />
			<div class="midtext">of</div>
			<input
				class="num"
				type="text"
				bind:value={info.disc_count}
				class:big={big(info.disc_count)}
			/>
		</div>
		<div class="row">
			<div class="label">Compilation</div>
			<p>{info.compilation ? 'Yes' : 'No'}</p>
		</div>
		<div class="row">
			<div class="label">Rating</div>
			<p>{info.rating}, {info.liked ? 'Liked' : 'Not Liked'}</p>
		</div>
		<div class="row">
			<div class="label">BPM</div>
			<input class="medium" type="text" bind:value={info.bpm} />
		</div>
		<div class="row">
			<div class="label">Play count</div>
			<p>{info.play_count}</p>
		</div>
		<div class="row">
			<div class="label">Comments</div>
			<input type="text" bind:value={info.comments} />
		</div>
		<div class="spacer"></div>
	</main>
	{#snippet buttons()}
		<Button secondary onclick={cancel}>Cancel</Button>
		<Button type="submit" onclick={() => save()}>Save</Button>
	{/snippet}
</Modal>

<style lang="sass">
	$cover-size: 90px
	.modal
		width: 450px
	.header
		display: flex
		align-items: stretch
		min-height: $cover-size
		&.has-subtitle
			margin-bottom: 7px
	.spacer
		height: 15px
	.text
		flex-grow: 1
		display: flex
		flex-direction: column
		justify-content: center
	.name
		font-size: 18px
		font-weight: 500
	.artist
		font-size: 13px
		opacity: 0.7
	.cover-area
		margin-right: 20px
		position: relative
		outline: none
		&:focus .outline-element
			box-shadow: 0px 0px 0px 2px var(--accent-1)
		&.droppable .outline-element
			box-shadow: 0px 0px 0px 2px var(--accent-1)
		&:focus.droppable .outline-element
			box-shadow: 0px 0px 0px 4px var(--accent-1)
	.cover
		transition: box-shadow 40ms ease-out
		width: $cover-size
		height: $cover-size
		display: flex
		align-items: center
		justify-content: center
		img
			display: block
			max-width: $cover-size
			max-height: $cover-size
		svg.cover-svg
			display: block
			padding: 26px
			width: $cover-size
			height: $cover-size
			box-sizing: border-box
			background: var(--empty-cover-bg-color)
			fill: var(--empty-cover-color)
			border-radius: 2px
	.cover-subtitle
		position: absolute
		font-size: 11px
		opacity: 0.8
		width: 100%
		margin-top: 1px
		text-align: center
		display: flex
		align-items: center
		justify-content: center
		.subtitle-text
			width: 36px
		.arrow
			margin-top: 1px
			display: flex
			color: #ffffff
			&.unclickable
				pointer-events: none
				svg
					opacity: 0.35
		svg
			background-color: transparent
			outline: none
			border: none
			padding: 0px
			opacity: 1
			transition: 0.05s ease-out
			&:active
				opacity: 0.7
				transform: scale(0.95)
	.row
		padding: 2px 0px
		display: flex
		align-items: center
		line-height: normal
		height: 27px
	input
		flex-grow: 1
		font-size: 13px
		padding: 3px 4px
		background-color: transparent
		color: inherit
		border: 1px solid rgba(#ffffff, 0.25)
		box-sizing: border-box
		&.num
			width: 30px
			flex-grow: 0
		&.big
			width: 50px
			flex-grow: 0
		&.medium
			width: 80px
			flex-grow: 0
		&:focus
			outline: 2px solid var(--accent-1)
			outline-offset: -1px
	p
		padding: 3px 0px
		font-size: 13px
		margin: 0px
	.label
		display: inline-block
		text-align: right
		width: 80px
		margin-right: 8px
		font-size: 12px
		opacity: 0.7
	.midtext
		display: inline-block
		width: 20px
		text-align: center
		line-height: normal
</style>
