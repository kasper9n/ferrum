// Based on kaleidosync code

import { create_singular_request_animation_frame } from '#lib/helpers.ts'
import { scaleLinear } from 'd3-scale'
import Meyda from 'meyda'
export type AudioStream = [number, number]
export type AudioStreamDefinitions = AudioStream[]

const visualizer_settings = {
	disableFlashing: false,
	def: [2.5, 0.09],
}

const BIT_DEPTH = Math.pow(2, 10)
const FILTER_TYPE = 'lowpass'
const FILTER_FREQUENCY = 7500
const FILTER_Q = 0.75

export function start_visualizer(
	audio_context: AudioContext,
	media_element_source: MediaElementAudioSourceNode,
	on_update: (info: { stream: number; volume: number }) => void,
) {
	const analyser = audio_context.createAnalyser()
	const filter = audio_context.createBiquadFilter()
	const current_rms_buffer = new Float32Array(BIT_DEPTH)
	const raf = create_singular_request_animation_frame()

	media_element_source.connect(filter)
	filter.connect(analyser)
	analyser.smoothingTimeConstant = 0
	analyser.fftSize = BIT_DEPTH
	filter.type = FILTER_TYPE
	filter.frequency.value = FILTER_FREQUENCY
	filter.Q.value = FILTER_Q

	let stream = 0
	let volume = 0

	const def = visualizer_settings?.def || [2.5, 0.09]
	const volume_history_buffer: number[] = []

	function sample_volume(total_samples: number) {
		let value = 0
		const end = Math.max(volume_history_buffer.length - 1, 0)
		const start = Math.max(end - total_samples, 0)
		total_samples = end - start + 1
		if (total_samples <= 0) {
			return [0, 0]
		}
		let min = Infinity
		for (let i = end; i >= start; i--) {
			value += volume_history_buffer[i]
			if (volume_history_buffer[i] < min) min = volume_history_buffer[i]
		}
		return [value / total_samples, min]
	}

	function measure_volume(frame_rate: number) {
		const raw_volume = get_raw_volume()
		volume_history_buffer.push(raw_volume)

		const [ref, min] = sample_volume((def[0] * 1000) / (1000 / frame_rate))
		const [sample] = sample_volume((def[1] * 1000) / (1000 / frame_rate))
		const scaled = scaleLinear([min, ref], [0, 1])(sample)
		// Make sure we don't do pow with a negative number
		const scaled_safe = Math.max(0, scaled)
		const raw = Number(Math.pow(scaled_safe, 1.5).toFixed(3))
		if (isNaN(raw)) {
			console.error('raw is NaN')
			return 1
		}
		return raw
	}

	function get_raw_volume() {
		analyser.getFloatTimeDomainData(current_rms_buffer)
		return (Meyda.extract('rms', current_rms_buffer) as number) || 0
	}

	let destroyed = false

	function measure() {
		// TODO: avoid hardcoded framerate
		const vol = measure_volume(60)

		if (visualizer_settings.disableFlashing) {
			volume = 1
		} else {
			volume = vol
		}

		const playing = audio_context.state === 'running'

		stream = stream + Math.pow(vol, 0.75) / (playing ? 10 : 100)

		on_update({ stream, volume })

		if (!destroyed) {
			raf_id = raf(measure)
		}
	}

	let raf_id = raf(measure)

	return {
		destroy() {
			destroyed = true
			if (raf_id) {
				cancelAnimationFrame(raf_id)
			}
			analyser.disconnect()
			filter.disconnect()
		},
	}
}
