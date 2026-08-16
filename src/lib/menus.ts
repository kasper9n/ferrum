import {
	add_tracks_to_playlist,
	get_track,
	get_track_list,
	get_track_playlist_ids,
	paths,
	track_lists_details,
} from '$lib/data.svelte'
import { flatten_child_lists } from '$lib/helpers'
import { ipc_renderer } from '$lib/window'
import type { TrackID } from 'ferrum-addon'
import { append_to_user_queue, prepend_to_user_queue } from './queue'
import type { SelectedTracksAction } from '$electron/typed_ipc'
import { open_track_info } from '$components/TrackInfo.svelte'
import { tracklist_actions } from './page'
import { tick } from 'svelte'
import { goto } from '$app/navigation'
import { resolve } from '$app/paths'

export function get_flattened_tracklists() {
	const track_lists = track_lists_details.map
	return flatten_child_lists(track_lists.root, track_lists, '')
}

export function get_tracklists_tree() {
	return get_flattened_tracklists()
}

export function get_show_in_playlists_tree(track_ids: TrackID[]) {
	if (track_ids.length !== 1) {
		return []
	}
	const matches = new Set(get_track_playlist_ids(track_ids[0]))
	return get_flattened_tracklists().filter((item) => {
		return item.enabled === false || matches.has(item.id)
	})
}

export function handle_selected_tracks_action({
	action,
	track_ids,
	all_ids,
	first_index,
}: {
	action: SelectedTracksAction
	track_ids: TrackID[]
	all_ids: TrackID[]
	first_index: number | null
}) {
	if (track_ids.length === 0 || first_index === null) {
		return
	}
	const first_track_id = track_ids[0]

	if (action === 'Play Next') {
		prepend_to_user_queue(track_ids)
	} else if (action === 'Add to Queue') {
		append_to_user_queue(track_ids)
	} else if (action === 'Get Info') {
		open_track_info(all_ids, first_index)
	} else if (action === 'reveal_track_file') {
		const track = get_track(first_track_id)
		ipc_renderer.invoke('revealTrackFile', paths.tracksDir, track.file)
	} else if (typeof action === 'object' && action.action === 'Add to Playlist') {
		add_tracks_to_playlist(action.playlist_id, track_ids)
	} else if (typeof action === 'object' && action.action === 'Show in Playlist') {
		let reveal_index = -1
		const list = get_track_list(action.playlist_id)
		if (list.type === 'playlist') {
			reveal_index = list.tracks.indexOf(first_track_id)
		}
		goto(resolve('/playlist/[id]', { id: action.playlist_id }))
		tick().then(() => {
			tracklist_actions.go_to_index(reveal_index)
		})
	}
}
