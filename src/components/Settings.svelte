<script lang="ts">
	import Modal from './Modal.svelte'
	import Button from './Button.svelte'
	import { save_view_options, view_options } from '$lib/data'

	interface Props {
		on_close: () => void
	}

	let { on_close }: Props = $props()

	let auto_update = $state(!view_options.noAutoUpdate)

	function save() {
		view_options.noAutoUpdate = !auto_update
		save_view_options(view_options)
		on_close()
	}
</script>

<Modal on_cancel={on_close} cancel_on_escape form={save} title="Settings">
	<div class="w-96 text-sm">
		<label class="flex items-center gap-2">
			<input
				type="checkbox"
				class="size-3.5 border-gray-300 bg-gray-100 text-blue-600 outline-offset-2 outline-blue-500 outline-solid focus-visible:outline"
				bind:checked={auto_update}
			/>
			Automatically check for updates on startup
		</label>
	</div>
	{#snippet buttons()}
		<Button secondary onclick={on_close}>Cancel</Button>
		<Button type="submit" onclick={save}>Save</Button>
	{/snippet}
</Modal>
