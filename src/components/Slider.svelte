<script lang="ts">
	import { onDestroy } from 'svelte'
	import type { HTMLAttributes } from 'svelte/elements'

	interface Props extends HTMLAttributes<HTMLDivElement> {
		value: number
		/** Growth rate per second. This is for providing a smooth visual with low CPU usage. */
		growth_rate?: number
		max?: number
		update_on_drag?: boolean
		on_user_change?: (value: number) => void
	}

	let {
		value = $bindable(),
		growth_rate = 0,
		max = 100,
		update_on_drag = true,
		on_user_change,
		class: klass,
		...rest
	}: Props = $props()

	let bar: HTMLDivElement | undefined = $state()
	let dragging = false

	let internal_updated_at = Date.now()
	let internal_value = $derived.by(() => {
		internal_updated_at = Date.now()
		return value
	})
	let drag_value: number | null = $state(null)
	let visual_value = $derived(drag_value ?? internal_value)

	function apply_growth() {
		const now = Date.now()
		const elapsed_ms = now - internal_updated_at
		internal_value = internal_value + elapsed_ms * 0.001 * growth_rate
		internal_updated_at = now
	}

	let interval: ReturnType<typeof setInterval> | null = null
	function start_growing() {
		if (interval) clearInterval(interval)
		if (bar && growth_rate > 0) {
			const secs_per_pixel = max / (bar.clientWidth * devicePixelRatio * 2 * growth_rate)
			interval = setInterval(apply_growth, secs_per_pixel * 1000)
		}
	}
	$effect(() => {
		start_growing()
	})

	function apply(e: MouseEvent) {
		if (!bar) {
			return
		}
		const delta = e.clientX - bar.getBoundingClientRect().left
		drag_value = Math.min(max, Math.max(0, (delta / bar.clientWidth) * max))
		if (!dragging || update_on_drag) {
			value = drag_value
			on_user_change?.(drag_value)
		}
	}

	onDestroy(() => {
		if (interval) clearInterval(interval)
	})
</script>

<svelte:window
	onmousemove={(e) => {
		if (dragging) {
			apply(e)
		}
	}}
	onmouseup={(e) => {
		if (dragging) {
			dragging = false
			apply(e)
			drag_value = null
		}
	}}
/>
<!-- If we had an <input>, it would cause a reflow every time the value updates. Instead of that, we CSS and mouse events. -->
<!-- We also don't use the Web Animation API, because somehow that had way higher CPU usage for me -->
<div class={['slider', klass]} {...rest}>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- Make sure it has enough padding for the thumb to not overflow -->
	<div
		class="group flex h-5 w-full items-center justify-center overflow-hidden p-2"
		onmousedown={(e) => {
			dragging = true
			apply(e)
		}}
	>
		<div class="pointer-events-none relative w-full rounded-full bg-gray-700" bind:this={bar}>
			<div class="w-full overflow-hidden rounded-full">
				<div
					class="relative -left-full h-1 w-full rounded-full bg-gray-300 transition-colors duration-100 will-change-transform group-hover:bg-[hsl(217,100%,60%)] group-active:bg-[hsl(217,100%,60%)]"
					style:translate="{(visual_value / max) * 100}%"
				></div>
			</div>
			<div
				class="absolute top-0 flex size-full items-center will-change-transform"
				style:translate="{(visual_value / max) * 100}%"
			>
				<div
					class="thumb size-2.5 -translate-x-[50%] scale-[0.4] rounded-full bg-gray-300 opacity-0 transition duration-75 group-hover:scale-100 group-hover:opacity-100 group-active:scale-100 group-active:opacity-100"
				></div>
			</div>
		</div>
	</div>
</div>

<style>
	@layer base {
		.slider {
			width: 129px;
		}
	}
	.thumb {
		box-shadow: 0px 0px 5px 1px rgba(0, 0, 0, 0.5);
	}
</style>
