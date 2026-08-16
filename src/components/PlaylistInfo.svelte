<script lang="ts">
	import { check_shortcut } from '../lib/helpers'
	import { new_playlist, type PlaylistInfo, update_playlist } from '$lib/data.svelte'
	import Modal from './Modal.svelte'
	import Button from './Button.svelte'

	interface Props {
		info: PlaylistInfo
		cancel: () => void
	}

	let { info = $bindable(), cancel }: Props = $props()

	function rows(value: string) {
		const matches = value.match(/\n/g) || []
		return Math.max(3, Math.min(matches.length + 1, 10))
	}

	function save() {
		if (info.editMode) {
			update_playlist(info.id, info.name, info.description)
		} else {
			new_playlist(info)
		}
		cancel()
	}

	function form_keydown(e: KeyboardEvent) {
		if (check_shortcut(e, 'enter', { cmd_or_ctrl: true })) {
			save()
		}
	}
</script>

<Modal
	on_cancel={cancel}
	cancel_on_escape
	form={save}
	onkeydown={form_keydown}
	title={(info.editMode ? 'Edit' : 'New') + ' Playlist' + (info.isFolder ? ' Folder' : '')}
>
	<main class="space-y-1">
		<input type="text" bind:value={info.name} placeholder="Title" />
		<textarea rows={rows(info.description)} bind:value={info.description} placeholder="Description"
		></textarea>
	</main>
	{#snippet buttons()}
		<Button secondary onclick={cancel}>Cancel</Button>
		<Button onclick={save}>Save</Button>
	{/snippet}
</Modal>

<style lang="sass">
	main
		display: flex
		flex-direction: column
	input, textarea
		resize: none
		line-height: normal
		width: 300px
		flex-grow: 1
		font-size: 13px
		font-family: inherit
		padding: 4px 6px
		background-color: transparent
		color: inherit
		border: 1px solid rgba(#ffffff, 0.25)
		box-sizing: border-box
		&:focus
			outline: 2px solid var(--accent-1)
			outline-offset: -1px
</style>
