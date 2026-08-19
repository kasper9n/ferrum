<script lang="ts">
	import { onDestroy } from 'svelte'
	import { useHistory } from '@reddojs/svelte'
	import { filter } from '$lib/data.svelte'
	import { ipc_listen } from '../lib/window'
	import type { HTMLInputAttributes } from 'svelte/elements'
	import { check_shortcut } from '$lib/helpers'

	const { execute, undo, redo } = useHistory({ size: 100 })

	let filter_input: HTMLInputElement | undefined = $state()

	type Snapshot = {
		value: string
		start: number
		end: number
		direction: SelectionDirection
	}

	let before: Snapshot | undefined

	function snapshot(): Snapshot {
		const input = filter_input!
		return {
			value: input.value,
			start: input.selectionStart ?? 0,
			end: input.selectionEnd ?? 0,
			direction: input.selectionDirection ?? 'none',
		}
	}

	function restore(s: Snapshot) {
		const input = filter_input!
		input.value = s.value
		filter.text = s.value
		input.setSelectionRange(s.start, s.end, s.direction)
	}

	function beforeinput(e: InputEvent) {
		if (e.inputType === 'historyUndo') {
			e.preventDefault()
			undo()
		} else if (e.inputType === 'historyRedo') {
			e.preventDefault()
			redo()
		} else {
			before = snapshot()
		}
	}

	function input(e: Event) {
		if (!before) return

		const event = e as InputEvent
		const after = snapshot()
		const previous = before
		before = undefined

		const undo_state: Snapshot =
			event.inputType.startsWith('delete') && previous.start === previous.end
				? {
						...previous,
						start: after.start,
						end: previous.start,
					}
				: previous

		// Only allow these three events to get combined into one undo action
		let key: string | undefined = event.inputType
		if (key !== 'insertText' && key !== 'deleteContentBackward' && key !== 'deleteContentForward') {
			key = undefined
		}

		execute({
			key,
			do: () => restore(after),
			undo: () => restore(undo_state),
		})

		filter.text = after.value
	}

	onDestroy(
		ipc_listen('filter', (e, text) => {
			if (text) {
				filter.text = text
				filter_input?.select()
			} else {
				filter_input?.select()
			}
		}),
	)

	interface Props {
		onfocus: HTMLInputAttributes['onfocus']
		onkeydown: HTMLInputAttributes['onkeydown']
	}

	let { onfocus, onkeydown }: Props = $props()
</script>

<input
	{onfocus}
	bind:this={filter_input}
	type="text"
	class="search rounded-[5px] text-[13px] leading-none"
	class:on={filter.text}
	value={filter.text}
	onbeforeinputcapture={beforeinput}
	oninput={input}
	onkeydown={(e) => {
		// on macOS, shift+cmd+z does not trigger onbeforeinput
		if (check_shortcut(e, 'z', { cmd_or_ctrl: true, shift: true })) {
			redo()
			e.preventDefault()
		}
		onkeydown?.(e)
	}}
	placeholder="Filter"
/>

<style lang="sass">
	input.search
		display: block
		width: calc(100% - 15px*2)
		margin: auto
		font-family: inherit
		padding: 5px 10px
		box-sizing: border-box
		color: inherit
		background-color: hsla(var(--hue), 68%, 90%, 0.08)
		outline: none
		border: 1px solid rgba(255, 255, 255, 0.1)
		&:focus
			background-color: hsla(var(--hue), 65%, 60%, 0.2)
			outline: 2px solid var(--accent-1)
			outline-offset: -1px
		&.on:focus
			outline: 2px solid hsl(160, 60%, 40%)
		&.on
			background-color: hsla(160, 65%, 60%, 0.15)
			border: 1px solid hsl(160, 50%, 60%, 0.2)
</style>
