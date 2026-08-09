<script lang="ts" module>
	export let drag_el: HTMLElement
	export let div: HTMLElement
	export function set_inner_text(new_text: string) {
		// eslint-disable-next-line svelte/no-dom-manipulating
		div.innerText = new_text
	}
</script>

<div class="drag-ghost" bind:this={drag_el}>
	<div bind:this={div}></div>
</div>

<style lang="sass">
	.drag-ghost
		font-size: 14px
		top: -1000px
		position: absolute
		background-color: transparent
		padding-left: 3px
		div
			background-color: var(--drag-bg-color)
			padding: 4px 8px
			max-width: 300px
			border-radius: 3px
</style>
