<script lang="ts">
	import { ItunesImport, track_lists_details_map, save, refreshers } from '$lib/data.svelte'
	import { ipc_renderer } from '$lib/window'
	import type { ImportStatus } from 'ferrum-addon'
	import Button from './Button.svelte'
	import Modal from './Modal.svelte'
	import { strict_call } from '$lib/error'

	interface Props {
		cancel: () => void
	}

	let { cancel }: Props = $props()
	let itunes_import = ItunesImport.new()

	type Stage = 'select' | 'fileSelect' | 'scanning' | ImportStatus
	let stage: Stage = $state('select')

	function cancel_handler() {
		if (stage === 'fileSelect' || stage === 'scanning') {
			return
		}
		cancel()
	}

	async function select_file() {
		stage = 'fileSelect'
		const open = await ipc_renderer.invoke('showOpenDialog', true, {
			properties: ['openFile'],
			filters: [{ name: 'iTunes Library File', extensions: ['xml'] }],
		})
		if (!open.canceled && open.filePaths[0]) {
			stage = 'scanning'
			const file_path = open.filePaths[0]
			stage = await strict_call(() => itunes_import.start(file_path))
		} else {
			stage = 'select'
		}
	}
	async function finish() {
		itunes_import.finish()
		save()
		refreshers.tracklist++
		track_lists_details_map.refresh()
		cancel()
	}
	async function submit() {
		if (stage === 'select') {
			select_file()
		} else if (typeof stage === 'object' && 'tracksCount' in stage) {
			finish()
		}
	}
</script>

<Modal on_cancel={cancel_handler} cancel_on_escape form={submit} title="Import iTunes Library">
	<main>
		{#if stage === 'select' || stage === 'fileSelect'}
			<p>
				Select an iTunes <strong>Library.xml</strong> file. To get that file, open iTunes and click
				on
				<strong>File > Library > Export Library...</strong>
			</p>
			<p>
				All your tracks need to be downloaded for this to work. If you have tracks from iTunes
				Store/Apple Music, it might not work.
			</p>
			<p>The following will not be imported:</p>
			<ul>
				<li>
					Lyrics, Equalizer, Skip when shuffling, Remember playback position, Start time, Stop time
				</li>
				<li>Album ratings, album likes and album dislikes</li>
				<li>The following track metadata:</li>
				<li>Music videos, podcasts, audiobooks, voice memos etc.</li>
				<li>Smart playlists, Genius playlists and Genius Mix playlists</li>
				<li>View options</li>
			</ul>
			<div class="buttons">
				<Button secondary onclick={cancel_handler}>Cancel</Button>
				<Button type="submit">Select File</Button>
			</div>
		{:else if stage === 'scanning'}
			Scanning...
		{:else if 'tracksCount' in stage}
			{#if stage.errors.length > 0}
				<div class="error-box">
					<h4>{stage.errors.length} Errors</h4>
					{#each stage.errors as error}
						<p>{error}</p>
					{/each}
				</div>
				<p>The following will be imported:</p>
			{:else}
				<p>Success, no errors! The following will be imported:</p>
			{/if}
			<ul>
				<li>Playlists: {stage.playlistsCount}</li>
				<li>Tracks: {stage.tracksCount}</li>
			</ul>
			<div class="buttons">
				<Button secondary onclick={cancel_handler}>Cancel</Button>
				<Button type="submit">Continue</Button>
			</div>
		{/if}
	</main>
</Modal>

<style lang="sass">
	main
		width: 530px
		line-height: 1.5
		display: flex
		flex-direction: column
	p, ul
		font-size: 0.95rem
		margin-top: 0px
	h4
		margin-block: 1em
	strong
		font-weight: normal
		background-color: hsl(0, 0%, 100%, 0.1)
		border: 1px solid hsl(0, 0%, 100%, 0.05)
		padding: 0.05em 0.25em
		border-radius: 3px
	.error-box
		background-color: hsla(0, 100%, 49%, 0.2)
		border: 1px solid hsl(0, 100%, 49%)
		border-radius: 5px
		padding: 0px 10px
		max-height: 500px
		overflow-y: scroll
		margin-bottom: 15px
	@media screen and (max-height: 800px)
		.error-box
			max-height: calc(100vh - 280px)
	@media screen and (max-height: 400px)
		.error-box
			max-height: 150px
	.buttons
		display: flex
		justify-content: flex-end
</style>
