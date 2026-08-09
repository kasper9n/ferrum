<script lang="ts">
	import type { SvelteComponent } from 'svelte'
	import { url_pathname } from './router'

	interface Props {
		route: string
		component: typeof SvelteComponent<Record<string, unknown>, Record<string, unknown>>
	}

	let { route, component }: Props = $props()

	function parse(pathname: string) {
		const params: Record<string, string> = {}
		const path_segments = pathname.split('/')
		if (route_segments.length !== path_segments.length) {
			return null
		}
		for (let i = 0; i < route_segments.length; i++) {
			const path_segment = path_segments[i]
			const route_segment = route_segments[i]
			if (route_segment.startsWith(':')) {
				params[route_segment.slice(1)] = path_segment
			} else if (path_segment !== route_segment) {
				return null
			}
		}
		return params
	}
	let route_segments = $derived(route.split('/'))
	let params = $derived(parse($url_pathname))
</script>

{#if params}
	{@const SvelteComponent_1 = component}
	<SvelteComponent_1 {params} />
{/if}
