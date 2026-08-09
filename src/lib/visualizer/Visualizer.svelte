<script lang="ts">
	import { start_visualizer } from './visualizer'
	import { shader_toy_lite, type ShaderToyLite } from './ShaderToyLite.js'
	import { onDestroy } from 'svelte'
	import { audio_context, media_element_source } from '$lib/player'
	import Player from '$components/Player.svelte'
	import { fade, fly } from 'svelte/transition'
	import { check_modifiers, check_shortcut } from '$lib/helpers'
	import { error_popup } from '$lib/error'
	import Button from '$components/Button.svelte'

	let { on_close }: { on_close: () => void } = $props()

	let vis_epilepsy_warning_dismissed = $state(
		localStorage.getItem('vis_epilepsy_warning_dismissed') === 'true',
	)
	function dismiss_vis_epilepsy_warning() {
		vis_epilepsy_warning_dismissed = true
		localStorage.setItem('vis_epilepsy_warning_dismissed', 'true')
	}

	const shaders = [
		{
			name: 'Color Warp',
			author: 'yozic',
			shader: `
				#define WARP true
				#define BALLS 10.
				#define CONTRAST 3
				#define GLOW .1
				#define ORB_SIZE 0.492519
				#define PI 3.14159265359

				vec2 kale(vec2 uv, vec2 offset, float splits) {
					float angle = atan(uv.y, uv.x);
					angle = ((angle / PI) + 1.0) * 0.5;
					angle = mod(angle, 1.0 / splits) * splits;
					angle = -abs(2.0 * angle - 1.0) + 1.0;
					float y = length(uv);
					angle = angle * (y);
					return vec2(angle, y) - offset;
				}

				void mainImage (out vec4 fragColor, in vec2 fragCoord) {
					vec2 uv = 2. * fragCoord/iResolution.xy - 1.;
					uv.x *= iResolution.x / iResolution.y;
					uv *= 2.2;
					fragColor = vec4(0.);
					float dist = distance(uv, vec2(0.));
					uv = WARP ? uv * kale(uv, vec2(0.), 2.) : uv;
					for (float i = 0.; i < BALLS; i++) {
						float t = iStream/2. - i * PI / BALLS * cos(iStream / max(i, 0.0001));
						vec2 p = vec2(cos(t), sin(t)) / sin(i / BALLS * PI / dist + iStream);
						vec3 c = iVolume * cos(vec3(0, 5, -5) * PI * 2. / PI + PI * (iStream / (i+1.) / 5.)) * (GLOW) + (GLOW);
						fragColor += vec4(iVolume * vec3(dist * .35 / length(uv - p * ORB_SIZE) * c), 1.0);
					}
					fragColor.xyz = pow(fragColor.xyz, vec3(CONTRAST));
				}
			`,
		},
		{
			name: 'Bars',
			author: 'yozic',
			shader: `
				#define orbs 20.
				#define PI 3.14159265359
				#define zoom 24.950
				#define div 10.000
				#define div2 10.000
				#define radius 8.159
				#define orbSize 0.682
				#define colorShift 12.710
				#define contrast 0.902

				// Helper function to get normalized UV coordinates
				vec2 k_uv() {
					vec2 uv = 2. * gl_FragCoord.xy/iResolution.xy - 1.;
					uv.x *= iResolution.x / iResolution.y;
					return uv;
				}

				// Helper function to create orb effect
				vec4 k_orb(vec2 uv, float size, vec2 position, vec3 color, float contrast_val) {
					float d = length(uv - position);
					float intensity = size / (d + 0.001);
					intensity = pow(intensity, contrast_val);
					return vec4(color * intensity, 1.0);
				}

				void mainImage (out vec4 fragColor, in vec2 fragCoord) {
					vec2 uv = k_uv();
					uv *= zoom;
					fragColor = vec4(0.);
					
					for (float i = 0.; i < orbs; i++) {
						uv.y -= cos((i+1.)*uv.y/div - iStream);
						uv.x += cos((i+1.)*uv.y/div2 - iStream/1.5);
						float t = i * PI / orbs;
						float x = radius * tan(t + iStream/2.);
						float y = radius * cos(t - iStream/2.2) * sin(t-iStream/3.);
						vec2 position = vec2(x, y);
						vec3 color = cos(vec3(-2, 0, -1) * PI * 2. / 3. + PI * (i / colorShift)) * 0.5 + 0.5;
						fragColor += k_orb(uv, iVolume * orbSize, position, color, contrast);
					}
				}
			`,
		},
		{
			name: 'Factory',
			author: 'yozic',
			shader: `
				#define BALLS 15.
				#define PI 3.14159265359
				#define zoom 47.379
				#define rotation 0.874
				#define yOuter 0.626
				#define yDivider 278.913
				#define xOuter 0.626
				#define xDivider 194.847
				#define multiplier 706.071
				#define radius 32.912
				#define ballSize 2.178
				#define contrast 2.529

				mat2 rotate2d(float _angle){
					return mat2(cos(_angle), -sin(_angle), sin(_angle), cos(_angle));
				}

				void mainImage (out vec4 fragColor, in vec2 fragCoord) {
					vec2 uv = 2. * fragCoord/iResolution.xy - 1.;
					uv.x *= iResolution.x / iResolution.y;
					uv *= zoom;
					fragColor = vec4(0.);
					
					uv = normalize(abs(uv) + sin(abs(uv) - iStream)) * length(uv);
					uv *= rotate2d(rotation);
					
					for (float i = 0.; i < BALLS; i++) {
						float dist = length(uv);
						uv.y += yOuter * sin(uv.y/yDivider + iStream/5.) / 1. * sin(uv.x/1. - iStream/3.);
						uv.x -= xOuter * sin(uv.x/xDivider - iStream/5.) / 1. * sin(uv.x/1.1 + iStream/1.);
						
						float t = i * PI / BALLS * (5. + 1.) + iStream/5000.;
						float _multiplier = dist * multiplier * sin(uv.x);
						vec2 p = vec2(radius * -2. * tan(t * multiplier), 2. * radius * sin(t * multiplier));
						
						vec3 col = cos(vec3(0, 1, -1) * PI * 2. / 3. + PI * (iStream / 20. + i / 5.)) * 0.5 + 0.5;
						fragColor += vec4(pow(iVolume, 1.3) * ballSize / length(uv - p * 0.9) * col, contrast);
					}
					
					fragColor.xyz = pow(fragColor.xyz, vec3(contrast));
					fragColor.w = 1.0;
				}
			`,
		},
		{
			name: 'Zippy Zaps',
			author: 'SnoopethDuckDuck',
			shader: `
				#define PI 3.14159265359
				#define GLOW 0.5

				vec2 stanh(vec2 a) {
					return tanh(clamp(a, -40., 40.));
				}

				void mainImage(out vec4 fragColor, in vec2 fragCoord) {
					vec2 v = iResolution.xy;
					vec2 u = 0.2 * (fragCoord.xy * 2. - v) / v.y;

					vec4 o = vec4(1, 2, 3, 0);
					vec4 col = vec4(0.0);

					for (float a = 0.5, t = iStream, i = 0.; i < 19.; i++) {
						v = cos(t + 1. - 7. * u * pow(a += 0.03, i)) - 5. * u;
						u += stanh(dot(u *= mat2(cos(i + 0.02 * t - vec4(0, 11, 33, 0))), u)
								* cos(1e2 * u.yx + t)) / 2e2
							+ 0.2 * a * u
							+ cos(4. / exp(dot(o, o) / 1e2) + t) / 3e2;
						t += 1.;
						col += (1. + cos(o + t)) / length((1. + i * dot(v, v)) * sin(1.5 * u / (0.5 - dot(u, u)) - 9. * u.yx + t));
					}

					col = 25.6 / (min(col, 13.) + 164. / col) - dot(u, u) / 250.;

					// Apply glow based on iVolume
					col.rgb *= pow(iVolume + 0.1, GLOW);

					fragColor = vec4(col.rgb, 1.0);
				}

			`,
		},
	]

	const image_shader = `
		void mainImage( out vec4 fragColor, in vec2 fragCoord ) {  
			vec2 uv = fragCoord.xy / iResolution.xy;
			vec4 col = texture(iChannel0, uv);
			fragColor = vec4(col.rgb, 1.);
		}
	`

	// main visualizer, and a transition visualizer
	type Vis = {
		is_main: boolean
		canvas?: HTMLCanvasElement
		toy?: ShaderToyLite
		should_resize?: boolean
	}
	const visualisers: [Vis, Vis] = $state([
		{
			id: 'vis-0',
			is_main: true,
		},
		{
			id: 'vis-1',
			is_main: false,
		},
	])
	let main_vis = visualisers[0]
	let next_vis = visualisers[1]

	let current_shader_index = 0
	let transition: {
		next_shader_index: number
		pending_shader_index: number | null
		auto: boolean
		duration: number
		start_time: number
		animation_old: Animation
		animation_new: Animation
		cancelled?: boolean
	} | null = null
	let auto_transition_timeout: ReturnType<typeof setTimeout> | undefined

	const visualizer = start_visualizer(audio_context, media_element_source, (info) => {
		main_vis.toy?.set_stream(info.stream)
		main_vis.toy?.set_volume(info.volume)
		// Also update transition toy if it exists
		next_vis.toy?.set_stream(info.stream)
		next_vis.toy?.set_volume(info.volume)
	})

	async function transition_to_shader(new_shader_index: number, auto: boolean) {
		if (!main_vis.canvas || !next_vis.canvas) {
			return console.error('Missing canvas')
		}

		if (transition) {
			const auto_to_manual = !auto && transition.auto
			const passed_half = Date.now() - transition.start_time > transition.duration * 0.5

			if (auto_to_manual) {
				// User wants to transition manually, but we're in a slow auto transition, so speed it up
				transition.auto = false
				transition.animation_new.updatePlaybackRate(4)
				transition.animation_old.updatePlaybackRate(4)
			}

			if (new_shader_index === transition.next_shader_index) {
				if (auto_to_manual && !passed_half) {
					// Auto transition already started here, but it's not halfway done, so we just speed it up
					transition.pending_shader_index = null
					return
				}
			}

			if (new_shader_index === current_shader_index) {
				if (!transition.cancelled) {
					transition.animation_new.reverse()
					transition.animation_old.reverse()
				}
				transition.cancelled = true
			}

			// Schedule the shader for later
			transition.pending_shader_index = new_shader_index
			// ...unless we're already transitioning to it
			if (transition.pending_shader_index === transition.next_shader_index) {
				transition.pending_shader_index = null
			}
			return
		}

		if (new_shader_index === current_shader_index) {
			return
		}

		clearTimeout(auto_transition_timeout)

		// Start transition visualizer
		if (!next_vis.toy) {
			const toy_result = shader_toy_lite(next_vis.canvas)
			if (!toy_result.data) {
				error_popup(toy_result.error)
				on_close()
				return
			}
			next_vis.toy = toy_result.data
			next_vis.toy.set_common('')
			next_vis.toy.set_image({ source: image_shader, iChannel0: 'A' })
		}
		if (next_vis.should_resize) {
			next_vis.should_resize = false
			schedule_resize(next_vis, true)
		}
		next_vis.toy.set_buffer_a({ source: shaders[new_shader_index].shader })
		if (!document.hidden) {
			next_vis.toy.play()
		}

		const duration = auto ? 2000 : 300

		const animation_new = next_vis.canvas.animate(
			{ opacity: [0, 1] },
			{
				duration: duration * 0.7,
				easing: 'cubic-bezier(0.45, 0, 0.55, 1)', // quadInOut
				fill: 'forwards',
			},
		)
		const animation_old = main_vis.canvas.animate(
			{ opacity: [1, 0] },
			{
				duration: duration * 0.7,
				delay: duration * 0.3,
				easing: 'cubic-bezier(0.45, 0, 0.55, 1)', // quadInOut
				fill: 'forwards',
			},
		)
		transition = {
			next_shader_index: new_shader_index,
			pending_shader_index: null,
			auto,
			duration,
			start_time: Date.now(),
			animation_new,
			animation_old,
		}
		await animation_new.finished
		await animation_old.finished

		if (transition.cancelled) {
			transition = null
			next_vis.toy?.pause() // Keep the instance for later use
			schedule_auto_transition()
			return
		}

		// Swap
		;[main_vis, next_vis] = [next_vis, main_vis]
		main_vis.is_main = true
		next_vis.is_main = false

		next_vis.toy?.pause() // Keep the instance for later use

		current_shader_index = transition?.next_shader_index

		if (transition.pending_shader_index !== null) {
			const new_index = transition.pending_shader_index
			transition = null
			transition_to_shader(new_index, false)
		} else {
			transition = null
			schedule_auto_transition()
		}
	}

	function schedule_auto_transition() {
		clearTimeout(auto_transition_timeout)
		auto_transition_timeout = setTimeout(() => {
			if (!transition) {
				const next_index = (current_shader_index + 1) % shaders.length
				transition_to_shader(next_index, true)
			}
		}, 20000)
	}

	schedule_auto_transition()

	let show_player = $state(false)
	let hide_player_timeout: ReturnType<typeof setTimeout> | undefined
	function show_player_temporarily() {
		show_player = true
		clearTimeout(hide_player_timeout)
		hide_player_timeout = setTimeout(() => {
			show_player = false
		}, 1000)
	}

	function schedule_resize(vis: Vis, now?: boolean) {
		if (!transition && !vis.is_main && !now) {
			vis.should_resize = true
			return
		}
		if (vis.canvas) {
			vis.canvas.width = window.innerWidth
			vis.canvas.height = window.innerHeight
			vis.toy?.resize()
		}
	}

	onDestroy(() => {
		clearTimeout(auto_transition_timeout)
		visualizer.destroy()
		main_vis.toy?.destroy()
		next_vis.toy?.destroy()
	})
</script>

<svelte:window
	onresize={() => {
		for (const vis of visualisers) {
			schedule_resize(vis)
		}
	}}
/>

<svelte:document
	onvisibilitychange={() => {
		if (document.hidden) {
			next_vis.toy?.pause()
			main_vis.toy?.pause()
		} else {
			main_vis.toy?.play()
			if (transition) {
				next_vis.toy?.play()
			}
		}
	}}
/>

<dialog
	class="max-h-screen max-w-screen bg-black outline-none"
	{@attach (e) => {
		e.showModal()
	}}
	onclose={() => {
		on_close()
	}}
	onkeydown={(e) => {
		if (check_modifiers(e, {})) {
			let shader_index = transition?.next_shader_index ?? current_shader_index
			if (e.key === 'ArrowLeft') {
				transition_to_shader((shader_index - 1 + shaders.length) % shaders.length, false)
			} else if (e.key === 'ArrowRight') {
				transition_to_shader((shader_index + 1) % shaders.length, false)
			} else if (/^[1-9]$/.test(e.key)) {
				transition_to_shader(Math.min(parseInt(e.key) - 1, shaders.length - 1), false)
			} else {
				show_player_temporarily()
			}
		}
	}}
>
	<div class="h-screen w-screen">
		{#each visualisers as vis}
			<canvas
				bind:this={vis.canvas}
				class="fixed inset-0"
				class:cursor-none={!show_player}
				style:z-index={vis.is_main ? '1' : '2'}
				class:mix-blend-screen={!vis.is_main}
				width={window.innerWidth}
				height={window.innerHeight}
				onmousemove={show_player_temporarily}
				{@attach (canvas) => {
					vis.canvas = canvas
					if (vis.is_main && !vis.toy) {
						const toy_result = shader_toy_lite(canvas)
						if (!toy_result.data) {
							error_popup(toy_result.error)
							on_close()
							return
						}
						const toy = toy_result.data
						toy.set_common('')
						toy.set_image({ source: image_shader, iChannel0: 'A' })
						toy.set_buffer_a({ source: shaders[0].shader })
						toy.play()
						vis.toy = toy
					}
				}}
			></canvas>
		{/each}
		{#if !vis_epilepsy_warning_dismissed}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="fixed inset-0 z-10 flex flex-col items-center justify-center gap-4 bg-black text-center text-lg text-white"
				onmousemove={show_player_temporarily}
				out:fade={{ duration: 200 }}
			>
				<h1 class="text-3xl font-medium">⚠️ Epilepsy Warning ⚠️</h1>
				<!-- tabindex+outline-none to prevent button autofocus -->
				<div
					class="flex outline-none"
					tabindex="-1"
					data-prevent-space-play
					onkeydown={(e) => {
						if (check_shortcut(e, ' ')) {
							e.preventDefault()
							on_close()
						} else if (check_shortcut(e, 'Enter')) {
							e.stopPropagation() // prevent player from revealing
							dismiss_vis_epilepsy_warning()
						}
					}}
				>
					<Button
						secondary
						onclick={() => {
							on_close()
						}}>Go back</Button
					>
					<Button
						onclick={() => {
							dismiss_vis_epilepsy_warning()
						}}>Continue</Button
					>
				</div>
			</div>
		{/if}
		{#if show_player}
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="fixed bottom-0 z-20 flex w-full flex-col justify-end"
				transition:fly={{ y: '100%', duration: 500 }}
				onmouseenter={() => {
					show_player = true
					clearTimeout(hide_player_timeout)
				}}
				onmouseleave={show_player_temporarily}
			>
				<Player on_toggle_visualizer={on_close} />
			</div>
		{/if}
	</div>
</dialog>
