<script lang="ts">
	import { run } from 'svelte/legacy'

	import { onDestroy } from 'svelte'
	import type { HTMLBaseAttributes } from 'svelte/elements'

	interface Props {
		value: number
		/** Growth rate per second. This is for providing a smooth visual with low CPU usage. */
		growth_rate?: number
		max?: number
		update_on_drag?: boolean
		on_user_change?: (value: number) => void
		class?: string
		step?: number
		[key: string]: any
	}

	let {
		value = $bindable(),
		growth_rate = 0,
		max = 100,
		update_on_drag = true,
		on_user_change = () => {},
		class: klass = '',
		...rest
	}: Props = $props()

	let bar: HTMLDivElement = $state()
	let dragging = $state(false)

	let internal_value = $state(value)
	let updated_at = $state(Date.now())
	run(() => {
		if (update_on_drag || !dragging) {
			internal_value = value
			updated_at = Date.now()
		}
	})

	function apply(e: MouseEvent) {
		const delta = e.clientX - bar.getBoundingClientRect().left
		internal_value = Math.min(max, Math.max(0, (delta / bar.clientWidth) * max))
		updated_at = Date.now()
		if (update_on_drag || !dragging) {
			value = internal_value
		}
	}

	function update_value() {
		const now = Date.now()
		const elapsed_ms = now - updated_at
		internal_value = internal_value + elapsed_ms * 0.001 * growth_rate
		updated_at = now
	}

	let interval: ReturnType<typeof setInterval> | null = null
	function start_growing() {
		if (interval) clearInterval(interval)
		const secs_per_pixel = max / (bar.clientWidth * devicePixelRatio * 2 * growth_rate)
		interval = setInterval(update_value, secs_per_pixel * 1000)
	}
	run(() => {
		if (growth_rate && bar) {
			start_growing()
		}
	})

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
			on_user_change(value)
		}
	}}
/>
<!-- If we had an <input>, it would cause a reflow every time the value updates. Instead of that, we CSS and mouse events. -->
<!-- We also don't use the Web Animation API, because somehow that had way higher CPU usage for me -->
<div class="slider{` ${klass}`.trimEnd()}" {...rest}>
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<!-- Make sure it has enough padding for the thumb to not overflow -->
	<div
		class="group flex h-5 w-full items-center justify-center overflow-hidden p-2"
		onmousedown={(e) => {
			apply(e)
			dragging = true
		}}
	>
		<div class="pointer-events-none relative w-full rounded-full bg-gray-700" bind:this={bar}>
			<div class="w-full overflow-hidden rounded-full">
				<div
					class="relative -left-full h-1 w-full rounded-full bg-gray-300 transition-colors duration-100 will-change-transform group-hover:bg-[hsl(217,100%,60%)] group-active:bg-[hsl(217,100%,60%)]"
					style:translate="{(internal_value / max) * 100}%"
				></div>
			</div>
			<div
				class="absolute top-0 flex size-full items-center will-change-transform"
				style:translate="{(internal_value / max) * 100}%"
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
