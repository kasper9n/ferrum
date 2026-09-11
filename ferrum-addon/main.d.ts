/* tslint:disable */
/* eslint-disable */

export * from './addon.d'

export type ItemId = number
export type TrackID = number
export type TrackListID = string
export type PercentInteger = number
export type MsSinceUnixEpoch = number

export interface Playlist {
	type: 'playlist'
}
export interface Folder {
	type: 'folder'
}
export interface Special {
	type: 'special'
}

export type TrackList = Playlist | Folder | Special
