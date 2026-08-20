<script lang="ts" module>
	import type { FilterTerm, Field } from 'ferrum-addon'
	class Filter {
		text = $state('')
		terms = $derived(parse_filter(this.text))
	}
	export const filter = new Filter()

	function is_whitespace(char: string) {
		return char === ' ' || char === '\t' || char === '\n' || char === '\r'
	}

	const fields: Record<string, Field | undefined> = {
		name: 0 satisfies Field.Title,
		title: 0 satisfies Field.Title,
		artist: 1 satisfies Field.Artist,
		band: 1 satisfies Field.Artist,
		album: 2 satisfies Field.Album,
		albumname: 2 satisfies Field.Album,
		album_name: 2 satisfies Field.Album,
		albumartist: 3 satisfies Field.AlbumArtist,
		album_artist: 3 satisfies Field.AlbumArtist,
		comment: 4 satisfies Field.Comments,
		comments: 4 satisfies Field.Comments,
		description: 4 satisfies Field.Comments,
		notes: 4 satisfies Field.Comments,
		genre: 5 satisfies Field.Genre,
		composer: 6 satisfies Field.Composer,
		group: 7 satisfies Field.Group,
		grouping: 7 satisfies Field.Group,
		year: 8 satisfies Field.Year,
		plays: 9 satisfies Field.Plays,
		skips: 10 satisfies Field.Skips,
		bpm: 11 satisfies Field.Bpm,
	}

	type FilterTermDetailed = FilterTerm & {
		field_text?: string
		literal_text?: string
	}
	function parse_filter(text: string) {
		const terms: FilterTermDetailed[] = []

		let i = 0

		while (i < text.length) {
			const word_start = i

			// Preserve whitespace
			while (is_whitespace(text[i])) i++
			if (word_start !== i) {
				terms.push({
					literal: text.slice(word_start, i),
				})
				continue
			}

			// "exact match" (end quote not necessary)
			if (text[i] === '"') {
				i++
				const value_start = i

				while (i < text.length && text[i] !== '"') i++

				const literal = text.slice(value_start, i)
				if (text[i] === '"') i++
				terms.push({
					literal_text: '"' + text.slice(value_start, i),
					literal,
				})
				console.log('terms', terms)
				continue
			}

			if (text[i] === ':') {
				// Word starting with ':' cannot be a field
				while (i < text.length && !is_whitespace(text[i])) i++
			} else {
				// Read the first word/field
				while (i < text.length && !is_whitespace(text[i]) && text[i] !== ':') i++
			}

			const field_text = text.slice(word_start, i)
			const field = fields[field_text.toLowerCase()]
			console.log(field, field_text)

			// field:value
			if (field !== undefined && text[i] === ':') {
				i++

				if (text[i] === '"') {
					// field:"multi word" (end quote not necessary)
					i++
					const value_start = i

					while (i < text.length && text[i] !== '"') i++

					const literal = text.slice(value_start, i)
					if (text[i] === '"') i++
					terms.push({
						field_text,
						field,
						literal_text: '"' + text.slice(value_start, i),
						literal,
					})
				} else {
					// field:value
					const value_start = i

					while (i < text.length && !is_whitespace(text[i])) i++

					const literal = text.slice(value_start, i)
					terms.push({
						field_text,
						field,
						literal,
					})
				}
			} else {
				const literal = text.slice(word_start, i)
				// Normal word
				terms.push({ literal })
			}
		}

		return terms
	}
</script>

<script lang="ts">
	import { onDestroy } from 'svelte'
	import { useHistory } from '@reddojs/svelte'
	import { ipc_listen } from '../lib/window'
	import type { HTMLInputAttributes } from 'svelte/elements'
	import { check_shortcut } from '$lib/helpers'

	interface Props {
		onfocus: HTMLInputAttributes['onfocus']
		onkeydown: HTMLInputAttributes['onkeydown']
	}
	let { onfocus, onkeydown }: Props = $props()

	const { execute, undo, redo } = useHistory({ size: 100 })

	let filter_input: HTMLInputElement | undefined = $state()

	type Snapshot = {
		value: string
		start: number
		end: number
		direction: SelectionDirection
	}

	let before: Snapshot | undefined

	// The last state represented by the history.
	// Do NOT obtain the "before" state from the DOM in the effect,
	// because Svelte may already have updated the DOM.
	let prev_known_state: Snapshot = {
		value: filter.text,
		start: 0,
		end: 0,
		direction: 'none',
	}

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

		filter.text = s.value
		prev_known_state = s
		input.value = s.value
		input.setSelectionRange(s.start, s.end, s.direction)
	}

	// Detect external updates to filter.text
	$effect(() => {
		const value = filter.text

		if (value === prev_known_state.value) return

		// DOM already got updated at this point, so use the saved previous state
		const previous = prev_known_state

		const after: Snapshot = {
			value,
			start: value.length,
			end: value.length,
			direction: 'none',
		}

		prev_known_state = after
		execute({
			key: undefined,
			do: () => restore(after),
			undo: () => restore(previous),
		})
	})

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

		let key: string | undefined = event.inputType
		if (key !== 'insertText' && key !== 'deleteContentBackward' && key !== 'deleteContentForward') {
			key = undefined
		}

		prev_known_state = after
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

	let overlay_el: HTMLDivElement | undefined = $state()

	function scroll_overlay() {
		if (overlay_el && filter_input) {
			overlay_el.scrollLeft = filter_input.scrollLeft
		}
	}
</script>

<div class="w-full px-3.5">
	<div class="relative">
		<div
			bind:this={overlay_el}
			class="fbox text-overlay pointer-events-none absolute top-0 z-10 overflow-hidden text-nowrap text-white"
			aria-hidden="true"
		>
			{#each filter.terms as term}
				{#if term.field !== undefined}
					{term.field_text}:<span class="highlight">{term.literal_text ?? term.literal}</span>
				{:else}
					{term.literal_text ?? term.literal}
				{/if}
			{/each}
		</div>

		<input
			{onfocus}
			bind:this={filter_input}
			type="text"
			class="fbox search relative rounded-[5px] text-transparent caret-white outline-none"
			value={filter.text}
			onbeforeinputcapture={beforeinput}
			oninput={input}
			onkeydown={(e) => {
				if (check_shortcut(e, 'z', { cmd_or_ctrl: true, shift: true })) {
					redo()
					e.preventDefault()
				}
				onkeydown?.(e)
			}}
			onscroll={scroll_overlay}
			placeholder="Filter"
		/>
	</div>
</div>

<style>
	:root {
		--filter-padding-x: 10px;
	}
	.fbox {
		display: block;
		border-radius: 5px;
		font-family: inherit;
		box-sizing: border-box;
		font-size: 13px;
		line-height: normal;
		padding-top: 5px;
		padding-bottom: 5px;
	}

	.text-overlay {
		border: 1px solid transparent;
		left: var(--filter-padding-x);
		right: var(--filter-padding-x);
	}

	.highlight {
		color: var(--icon-highlight-color);
	}

	.search {
		width: 100%;
		padding-left: var(--filter-padding-x);
		padding-right: var(--filter-padding-x);
		background-color: hsla(var(--hue), 68%, 90%, 0.08);
		border: 1px solid rgba(255, 255, 255, 0.1);
		&::placeholder {
			color: rgba(255, 255, 255, 0.5);
		}
		&:focus {
			background-color: hsla(var(--hue), 65%, 60%, 0.2);
			outline: 2px solid var(--accent-1);
			outline-offset: -1px;
		}
	}
</style>
