<script lang="ts">
	import { filter, get_artists } from '$lib/data.svelte'
	import fuzzysort from 'fuzzysort'
	import Header from '$components/Header.svelte'

	let all_artists = get_artists()
	let snapshot_artists = $derived(fuzzysort.snapshot(all_artists))
	let artists = $derived(fuzzysort.go(filter.text, snapshot_artists, { limit: 0 }))
</script>

<Header title="Artists" subtitle="{all_artists.length} artists" description={undefined} />
<div class="w-full border-b border-b-slate-500/30">
	<p class="px-3">(Work in progress)</p>
</div>

<div class="size-full overflow-y-auto text-sm">
	{#each artists as artist}
		<p class="block px-3 py-1 text-current">
			{#if artist.target}
				{artist.target}
			{:else}
				Unknown Artist
			{/if}
		</p>
	{/each}
</div>
