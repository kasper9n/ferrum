import type { TrackID } from 'ferrum-addon'

type DraggedTracks = {
	ids: TrackID[]
	playlist_indexes?: number[]
	queue_indexes?: number[]
}
type DraggedPlaylist = {
	id: string
	from_folder: string
	level: number
}

type Dragged = {
	tracks?: DraggedTracks
	playlist?: DraggedPlaylist
}
export const dragged: Dragged = {}
