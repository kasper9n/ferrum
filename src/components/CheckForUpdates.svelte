<script lang="ts">
	import { ipc_renderer } from '$lib/window'
	import Modal from './Modal.svelte'
	import Button from './Button.svelte'
	import { is_dev, save_view_options, view_options } from '$lib/data'
	import { check_shortcut } from '$lib/helpers'

	let latest_update: Awaited<ReturnType<typeof check>> | null = $state(null)

	check()
	async function check() {
		if (is_dev || !window.navigator.onLine || view_options.noAutoUpdate) {
			return
		}
		const result = await ipc_renderer.invoke('check_for_updates')
		if (
			!result ||
			result.channel.version === result.app_version ||
			result.channel.version === view_options.skipUpdatingToVersion
		) {
			return
		}

		latest_update = result
		return result
	}

	function skip_update(version: string) {
		view_options.skipUpdatingToVersion = version
		save_view_options(view_options)
	}
</script>

{#if latest_update}
	{@const channel = latest_update.channel}
	<Modal
		on_cancel={() => (latest_update = null)}
		cancel_on_escape
		form={() => {
			ipc_renderer.invoke('open_url', channel.url)
			latest_update = null
		}}
		title="A new version of Ferrum is available!"
		on:keydown={(e) => {
			if (check_shortcut(e, 'Enter')) {
				ipc_renderer.invoke('open_url', channel.url)
				latest_update = null
			}
		}}
	>
		<!-- svelte-ignore a11y_autofocus -->
		<div class="w-md max-w-xl text-sm outline-none" autofocus tabindex="-1">
			<p class="pb-3">
				Ferrum {latest_update.channel.version} is available. You are currently on {latest_update.app_version}
			</p>
			<p class="pb-1 font-semibold">Release notes:</p>
			<p class="max-h-72 overflow-y-auto rounded bg-black/30 px-3 py-2 whitespace-pre-wrap">
				{latest_update.body}
			</p>
		</div>
		{#snippet buttons()}
			<Button
				secondary
				onclick={() => {
					skip_update(channel.version)
					latest_update = null
				}}>Skip This Version</Button
			>
			<div class="grow"></div>
			<Button secondary onclick={() => (latest_update = null)}>Later</Button>
			<Button type="submit">Update</Button>
		{/snippet}
	</Modal>
{/if}
