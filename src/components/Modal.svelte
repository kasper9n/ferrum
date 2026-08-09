<script lang="ts" module>
	import { writable } from 'svelte/store'

	export const modal_count = writable(0)
</script>

<script lang="ts">
	import { self, createBubbler, handlers, preventDefault } from 'svelte/legacy'

	const bubble = createBubbler()
	import { onDestroy, onMount } from 'svelte'
	import { check_shortcut } from '../lib/helpers'

	interface Props {
		on_cancel: () => void
		cancel_on_escape?: boolean
		form?: (() => void) | undefined
		plain?: boolean
		title?: string | null
		children?: import('svelte').Snippet
		buttons?: import('svelte').Snippet
	}

	let {
		on_cancel,
		cancel_on_escape = false,
		form = undefined,
		plain = false,
		title = null,
		children,
		buttons,
	}: Props = $props()
	let dialog_el: HTMLDialogElement = $state()

	let last_focused_el: Element | undefined | null

	$modal_count += 1
	onDestroy(() => {
		$modal_count -= 1
		dialog_el.close()
		if (last_focused_el instanceof HTMLElement) {
			// For some reason necessary with Svelte 5, maybe onDestroy runs too early
			last_focused_el?.focus()
		}
	})

	onMount(() => {
		last_focused_el = document.activeElement
		dialog_el.showModal()
	})

	// Prevent clicks where the mousedown or mouseup happened on a child element. This could've
	// been solved with a non-parent backdrop element, but that interferes with text selection.
	let clickable = $state(true)
	let tag = $derived(form === undefined ? 'div' : 'form')
</script>

<svelte:body
	onclick={() => {
		clickable = true
	}}
/>

<div class="dragbar absolute top-0 left-0 w-full"></div>

<dialog
	class="modal m-auto"
	bind:this={dialog_el}
	tabindex="-1"
	onclick={self(() => {
		if (clickable) {
			on_cancel()
		}
	})}
	onkeydown={handlers(
		bubble('keydown'),
		(e) => {
			if (check_shortcut(e, 'Escape')) {
				e.preventDefault()
				if (cancel_on_escape) {
					on_cancel()
				}
			}
		},
		self((e) => {
			if (form && check_shortcut(e, 'Enter')) {
				form()
				e.preventDefault()
			}
		}),
	)}
>
	<svelte:element
		this={tag}
		class="box"
		class:padded={!plain}
		onsubmit={preventDefault(form)}
		onmousedown={() => {
			clickable = false
		}}
		onmouseup={() => {
			clickable = false
		}}
		role="none"
	>
		{#if title !== null}
			<h3 class="text-lg">{title}</h3>
		{/if}
		{@render children?.()}
		{#if buttons}
			<div class="buttons">
				{@render buttons?.()}
			</div>
		{/if}
	</svelte:element>
</dialog>

<style lang="sass">
	.dragbar
		-webkit-app-region: drag
		height: 25px
	h3
		font-weight: 500
		margin-bottom: 0.875rem
	::backdrop
		background-color: rgba(#000000, 0.4)
	dialog
		max-height: calc(100% - 40px)
		color: inherit
		box-sizing: border-box
		box-shadow: 0px 0px 30px 0px rgba(#000000, 0.5)
		background-color: rgba(#1b1d22, 75%)
		backdrop-filter: saturate(3) blur(20px) brightness(1.25)
		padding: 0
		border: 1px solid rgba(#ffffff, 0.2)
		border-radius: 7px
	.padded
		padding: 1.125rem
		.buttons
			margin-top: 1.125rem
	.buttons
		display: flex
		justify-content: flex-end
</style>
