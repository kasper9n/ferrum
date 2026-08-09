<script lang="ts">
	import { get_track, join_paths, paths, tracks_updated } from '$lib/data'
	import { onDestroy } from 'svelte'

	interface Props {
		id: string
	}

	let { id }: Props = $props()

	let track = $derived(get_track(id))
	onDestroy(
		tracks_updated.subscribe(() => {
			track = get_track(id)
		}),
	)

	let src = $derived(
		'app://trackimg?path=' +
			encodeURIComponent(join_paths(paths.tracksDir, track.file)) +
			'&cache_db_path=' +
			encodeURIComponent(paths.cacheDb) +
			'&date_modified=' +
			encodeURIComponent(track.dateModified),
	)

	let failed_src: string | null = $state(null)
	let loaded = $state(false)
</script>

<div class="box">
	{#if src === failed_src && failed_src !== null}
		<svg
			class="cover"
			xmlns="http://www.w3.org/2000/svg"
			width="24"
			height="24"
			viewBox="0 0 24 24"
		>
			<path
				d="M23 0l-15.996 3.585v13.04c-2.979-.589-6.004 1.671-6.004 4.154 0 2.137 1.671 3.221 3.485 3.221 2.155 0 4.512-1.528 4.515-4.638v-10.9l12-2.459v8.624c-2.975-.587-6 1.664-6 4.141 0 2.143 1.715 3.232 3.521 3.232 2.14 0 4.476-1.526 4.479-4.636v-17.364z"
			/>
		</svg>
	{:else}
		<img
			class="cover"
			class:invisible={!loaded}
			{src}
			alt=""
			onload={() => {
				loaded = true
				failed_src = null
			}}
			onerror={() => {
				failed_src = src
			}}
		/>
	{/if}
</div>
<div class="text">
	<p>{track.name}</p>
	<p class="artist">{track.artist}</p>
</div>

<style lang="sass">
	.text
		overflow: hidden
		white-space: nowrap
		line-height: normal
	p
		margin: 0px
		font-size: 14px
		overflow: hidden
		text-overflow: ellipsis
	.artist
		opacity: 0.75
		font-size: 12px
	.box
		margin-right: 10px
	.cover, .box
		width: 42px
		min-width: 42px
		height: 42px
		min-height: 42px
		pointer-events: none
	.invisible
		opacity: 0
	img
		object-fit: contain
	svg
		padding: 12px
		box-sizing: border-box
		background: var(--empty-cover-bg-color)
		fill: var(--empty-cover-color)
		border-radius: 2px
</style>
