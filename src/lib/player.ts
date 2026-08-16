import { derived, get, writable } from 'svelte/store'
import type { Writable } from 'svelte/store'
import { clamp_number } from './helpers'
import quit from './quit'
import {
	add_play,
	add_play_time,
	add_skip,
	get_track,
	get_track_ids,
	join_paths,
	paths,
	read_cover_async,
} from '$lib/data.svelte'
import type { Track, TrackID } from 'ferrum-addon'
import { ipc_renderer } from './window'
import { queue, set_new_queue, next as queueNext, prev as queuePrev } from './queue'
import { tracks_page_item_ids } from '$routes/playlist/[id]/+page.svelte'

const audio = new Audio()
export const audio_context = new AudioContext()
export const media_element_source = audio_context.createMediaElementSource(audio)
const gain_node = audio_context.createGain()
media_element_source.connect(gain_node)
gain_node.connect(audio_context.destination)

let is_stopped = true
export const stopped = (() => {
	const { subscribe, set } = writable(true)
	return {
		subscribe,
		set: (value: boolean) => {
			is_stopped = value
			set(value)
		},
	}
})()
export const time_record = writable({
	elapsed: 0,
	at_timestamp: Date.now(),
	paused: true,
	duration: 0,
})
function update_time_details() {
	time_record.update((record) => {
		if (Number.isNaN(audio.duration)) {
			record.elapsed = 0
			record.duration = 0
		} else {
			record.elapsed = audio.currentTime
			record.duration = audio.duration
		}
		record.at_timestamp = Date.now()
		record.paused = audio.paused
		return record
	})
}
export const playing_track: Writable<Track | null> = writable(null)
export const playing_id = derived(queue, () => {
	const current_id = queue.getCurrent()?.id
	return current_id
})
let waiting_to_play = false
const media_session = navigator.mediaSession

export const volume = (() => {
	let last_volume = 1
	const store = writable(1)
	function set(value: number) {
		last_volume = gain_node.gain.value
		gain_node.gain.value = clamp_number(0, 1, value)
		store.set(clamp_number(0, 1, value))
	}
	return {
		set,
		toggle() {
			if (gain_node.gain.value > 0) set(0)
			else set(last_volume || 1)
		},
		subscribe: store.subscribe,
	}
})()
ipc_renderer.on('volumeUp', () => {
	volume.set(gain_node.gain.value + 0.05)
})
ipc_renderer.on('volumeDown', () => {
	volume.set(gain_node.gain.value - 0.05)
})

export const cover_src = (() => {
	const { set, subscribe }: Writable<string | null> = writable(null)
	return {
		async newFromTrackId(id: TrackID) {
			try {
				const file_path = join_paths(paths.tracksDir, get_track(id).file)
				const result = await read_cover_async(file_path, 0)
				if (!result) {
					set(null)
				} else {
					// this should be zero-copy
					const uint8 = new Uint8Array(result)
					// Blob() does copy
					const url = URL.createObjectURL(new Blob([uint8], {}))
					set(url)
				}
			} catch (_) {
				set(null)
			}
		},
		subscribe,
	}
})()
playing_id.subscribe((current_id) => {
	if (current_id) {
		cover_src.newFromTrackId(current_id)
	}
})

let start_time: number | null = null
function get_play_time() {
	return Date.now() - (start_time ?? Date.now())
}

/** Saves play time if needed */
function reset_and_save_play_time() {
	const current_id = queue.getCurrent()?.id
	if (get_play_time() >= 1000 && current_id && start_time) {
		add_play_time(current_id, start_time, get_play_time())
	}
	start_time = null
}

audio.oncanplay = () => {
	if (waiting_to_play) {
		waiting_to_play = false
		start_playback()
	}
}
audio.onplay = update_time_details
audio.onplaying = () => {
	start_time = Date.now()
	update_time_details()
}
audio.onloadeddata = update_time_details
audio.onloadedmetadata = update_time_details
audio.onpause = () => {
	update_time_details()
	reset_and_save_play_time()
}
audio.onwaiting = () => {
	reset_and_save_play_time()
}
audio.onemptied = () => {
	reset_and_save_play_time()
}
audio.onstalled = () => {
	reset_and_save_play_time()
}
audio.onended = () => {
	reset_and_save_play_time()
	next(false)
}
audio.onerror = async () => {
	reset_and_save_play_time()
	stop()
	let message = 'Audio playback error'
	let detail = 'Unknown error'
	if (audio && audio.error) {
		detail = audio.error.message
		if (audio.error.code === audio.error.MEDIA_ERR_SRC_NOT_SUPPORTED) {
			if (audio.networkState === audio.NETWORK_NO_SOURCE) {
				message = 'File not found'
				detail = ''
			}
		}
	}
	await ipc_renderer.invoke('showMessageBox', false, { type: 'error', message, detail })
}
audio.ontimeupdate = update_time_details
audio.ondurationchange = update_time_details

function start_playback() {
	audio.play()
	stopped.set(false)
	if (media_session) media_session.playbackState = 'playing'
}

function set_playing_file(id: TrackID, paused = false) {
	const track = get_track(id)
	const file_url = 'track:' + join_paths(paths.tracksDir, track.file)
	waiting_to_play = !paused
	audio.src = file_url
	playing_track.set(track)
	if (media_session) {
		media_session.metadata = new MediaMetadata({
			title: track.name,
			artist: track.artist,
			album: track.albumName,
			// artwork: [{ src: 'podcast.jpg' }],
		})
	}
}

function pause_playback() {
	waiting_to_play = false
	audio.pause()
	if (media_session) media_session.playbackState = 'paused'
}

export function new_playback_instance(new_queue: TrackID[], index: number) {
	if (!is_stopped) pause_playback()
	set_new_queue(new_queue, index)
	const current = queue.getCurrent()
	if (current) {
		set_playing_file(current.id)
	}
}

export function play_pause() {
	if (is_stopped) {
		if (get(tracks_page_item_ids).length > 0) {
			const all_track_ids = get_track_ids(get(tracks_page_item_ids))
			new_playback_instance(all_track_ids, 0)
		}
	} else if (audio.paused) {
		start_playback()
	} else {
		pause_playback()
	}
}

export function reload() {
	const id = queue.getCurrent()?.id
	const was_paused = audio.paused
	if (id && !is_stopped) {
		const current_time = audio.currentTime
		audio.src = ''
		audio.load()
		set_playing_file(id, was_paused)
		audio.currentTime = current_time
	}
}

export function stop() {
	waiting_to_play = false
	audio.pause()
	stopped.set(true)
	seek(0)
	if (media_session) {
		media_session.playbackState = 'none'
		media_session.metadata = null
	}
}

quit.set_handler('player', () => {
	stop()
})

export function skip_to_next() {
	next(true)
}

function next(skip: boolean) {
	const current_id = queue.getCurrent()?.id
	if (current_id) {
		if (skip) {
			add_skip(current_id)
		} else {
			add_play(current_id)
		}
		queueNext()
		const new_current_id = queue.getCurrent()?.id
		if (new_current_id) {
			set_playing_file(new_current_id)
		} else {
			stop()
		}
	}
}
export function previous() {
	const current_id = queue.getCurrent()?.id
	if (current_id) {
		queuePrev()
		const new_current_id = queue.getCurrent()?.id
		if (new_current_id) {
			set_playing_file(new_current_id)
		} else {
			// this should never happen because the first track will play again
			stop()
		}
	}
}

export function seek(to: number, fast_seek = false) {
	const new_time = Math.min(to, audio.duration || 0)
	if (fast_seek && audio.fastSeek) {
		audio.fastSeek(to)
	} else {
		audio.currentTime = new_time
	}
	update_time_details()
}

if (navigator.mediaSession) {
	const media_sesison = navigator.mediaSession
	media_sesison.setActionHandler('play', start_playback)
	media_sesison.setActionHandler('pause', pause_playback)
	media_sesison.setActionHandler('stop', stop)
	media_sesison.setActionHandler('seekbackward', (details) => {
		seek(audio.currentTime - (details.seekOffset || 5))
	})
	media_sesison.setActionHandler('seekforward', (details) => {
		seek(audio.currentTime + (details.seekOffset || 5))
	})
	media_sesison.setActionHandler('seekto', (details) => {
		if (details.seekTime && details.fastSeek) {
			seek(details.seekTime, details.fastSeek)
		}
	})
	media_sesison.setActionHandler('previoustrack', previous)
	media_sesison.setActionHandler('nexttrack', skip_to_next)
}

ipc_renderer.on('playPause', play_pause)
ipc_renderer.on('Next', skip_to_next)
ipc_renderer.on('Previous', previous)
ipc_renderer.on('Stop', stop)
