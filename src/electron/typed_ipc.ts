import type {
	IpcMain as ElectronIpcMain,
	IpcMainEvent,
	IpcMainInvokeEvent as ElectronIpcMainInvokeEvent,
	IpcRenderer as ElectronIpcRenderer,
	IpcRendererEvent,
	WebContents as ElectronWebContents,
	dialog,
	MenuItemConstructorOptions,
} from 'electron'
import { ipcMain as electronIpcMain } from 'electron'
import type { TrackID } from '../../ferrum-addon'
import type { UpdateJson } from './update'

type InputMap = {
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	[key: string]: (...args: any[]) => any
}

interface TypedWebContents<IpcEvents extends InputMap> extends ElectronWebContents {
	send<K extends keyof IpcEvents>(channel: K, ...args: Parameters<IpcEvents[K]>): void
}

interface TypedIpcMainInvokeEvent<IpcEvents extends InputMap> extends ElectronIpcMainInvokeEvent {
	sender: TypedWebContents<IpcEvents>
}

interface TypedIpcMain<
	IpcEvents extends InputMap,
	IpcCommands extends InputMap,
> extends ElectronIpcMain {
	on<K extends keyof IpcEvents>(
		channel: K,
		listener: (event: IpcMainEvent, ...args: Parameters<IpcEvents[K]>) => void,
	): this
	once<K extends keyof IpcEvents>(
		channel: K,
		listener: (event: IpcMainEvent, ...args: Parameters<IpcEvents[K]>) => void,
	): this
	removeListener<K extends keyof IpcEvents>(
		channel: K,
		listener: (event: IpcMainEvent, ...args: Parameters<IpcEvents[K]>) => void,
	): this
	removeAllListeners<K extends keyof IpcEvents>(channel?: K): this
	handle<K extends keyof IpcCommands>(
		channel: K,
		listener: (
			event: TypedIpcMainInvokeEvent<IpcEvents>,
			...args: Parameters<IpcCommands[K]>
		) => ReturnType<IpcCommands[K]>,
	): void
	handleOnce<K extends keyof IpcCommands>(
		channel: K,
		listener: (
			event: TypedIpcMainInvokeEvent<IpcEvents>,
			...args: Parameters<IpcCommands[K]>
		) => ReturnType<IpcCommands[K]>,
	): void
	removeHandler<K extends keyof IpcCommands>(channel: K): void
}

interface TypedIpcRenderer<
	IpcEvents extends InputMap,
	IpcCommands extends InputMap,
> extends ElectronIpcRenderer {
	on<K extends keyof IpcEvents>(
		channel: K,
		listener: (event: IpcRendererEvent, ...args: Parameters<IpcEvents[K]>) => void,
	): this
	once<K extends keyof IpcEvents>(
		channel: K,
		listener: (event: IpcRendererEvent, ...args: Parameters<IpcEvents[K]>) => void,
	): this
	removeListener<K extends keyof IpcEvents>(
		channel: K,
		listener: (event: IpcRendererEvent, ...args: Parameters<IpcEvents[K]>) => void,
	): this
	removeAllListeners<K extends keyof IpcEvents>(channel?: K): this
	send<K extends keyof IpcEvents>(channel: K, ...args: Parameters<IpcEvents[K]>): void
	sendSync<K extends keyof IpcEvents>(
		channel: K,
		...args: Parameters<IpcEvents[K]>
	): ReturnType<IpcEvents[K]>
	sendTo<K extends keyof IpcEvents>(
		webContentsId: number,
		channel: K,
		...args: Parameters<IpcEvents[K]>
	): void
	sendToHost<K extends keyof IpcEvents>(channel: K, ...args: Parameters<IpcEvents[K]>): void
	invoke<K extends keyof IpcCommands>(
		channel: K,
		...args: Parameters<IpcCommands[K]>
	): Promise<ReturnType<IpcCommands[K]>>
}

export type SelectedTracksAction =
	| 'Play Next'
	| 'Add to Queue'
	| { action: 'Add to Playlist'; playlist_id: string }
	| { action: 'Show in Playlist'; playlist_id: string }
	| 'Get Info'
	| 'reveal_track_file'
	| 'Remove from Playlist'
	| 'Delete from Library'

type Events = {
	readyToQuit: () => void
	gonnaQuit: () => void

	newPlaylist: (id: string, isFolder: boolean) => void
	show_settings: () => void
	itunesImport: () => void
	import: () => void
	filter: (text?: string) => void

	playPause: () => void
	Next: () => void
	Previous: () => void
	Stop: () => void
	Shuffle: () => void
	Repeat: () => void
	volumeUp: () => void
	volumeDown: () => void
	'Show Queue': () => void
	'Show Lyrics': () => void
	'Toggle Visualizer': () => void
	ToggleQuickNav: () => void
	'Group Album Tracks': (checked: boolean) => void

	selected_tracks_action: (action: SelectedTracksAction) => void

	Back: () => void
	Forward: () => void
	'Select Next List': () => void
	'Select Previous List': () => void

	'context.playlist.edit': (id: TrackID) => void
	'context.playlist.delete': (id: TrackID) => void
	'context.Remove from Queue': () => void
	'context.Get Info': (allIds: TrackID[], selectedIndex: number) => void
	'context.Remove from Playlist': (selectedIndexes: number[]) => void
	'context.toggle_column': (item: { id: string; label: string; checked: boolean }) => void
}

export type ShowTrackMenuOptions = {
	lists: TrackMenuNode[]
	show_in_playlists: TrackMenuNode[]
	is_editable_playlist: boolean
	queue: boolean
}

export type TrackMenuNode = {
	label: string
	enabled: boolean
	id: string
}

type Commands = {
	app_loaded: () => void
	check_for_updates: () => Promise<{
		channel: UpdateJson[string]
		app_version: string
		body: string
	} | null>
	open_url: (url: string) => void
	showMessageBox: (
		attached: boolean,
		options: Parameters<typeof dialog.showMessageBox>[0],
		crash?: boolean,
	) => ReturnType<typeof dialog.showMessageBox>
	showOpenDialog: (
		attached: boolean,
		options: Parameters<typeof dialog.showOpenDialog>[0],
	) => ReturnType<typeof dialog.showOpenDialog>
	revealTrackFile: (...paths: string[]) => void
	show_tracks_menu: (options: ShowTrackMenuOptions) => Promise<null | SelectedTracksAction>
	showTracklistMenu: (options: { id: string; isFolder: boolean; isRoot: boolean }) => void
	show_columns_menu: (options: {
		column_filter: string | null
		menu: MenuItemConstructorOptions[]
	}) => void
	volume_change: (up: boolean) => void
	init_media_keys: (
		prompt: boolean,
	) => Promise<
		| { needs_accessibility_permission: undefined; error: string }
		| { needs_accessibility_permission: boolean; error: undefined }
		| null
	>

	'update:Shuffle': (checked: boolean) => void
	'update:Repeat': (checked: boolean) => void
	'update:Show Queue': (checked: boolean) => void
	'update:Show Lyrics': (checked: boolean) => void
}

export type IpcMain = TypedIpcMain<Events, Commands>
export type IpcRenderer = TypedIpcRenderer<Events, Commands>
export type WebContents = TypedWebContents<Events>

export const ipc_main = electronIpcMain as IpcMain

interface TypedIpcFunctions<IpcEvents extends InputMap> extends ElectronIpcRenderer {
	ipcListen<K extends keyof IpcEvents>(
		channel: K,
		listener: (event: IpcRendererEvent, ...args: Parameters<IpcEvents[K]>) => void,
	): () => void
}
export type IpcFunctions = TypedIpcFunctions<Events>
