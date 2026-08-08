import { type App, BrowserWindow, Menu, shell, dialog } from 'electron'
import { ipc_main } from './typed_ipc'
import type { MenuItemConstructorOptions } from 'electron/common'
import is from './is'
import type { WebContents } from './typed_ipc'

export async function handle_missing(id: string) {
	dialog.showMessageBoxSync({
		type: 'error',
		message: 'Missing ID ' + id,
		detail: 'Please let me know what happened',
		title: 'Error',
	})
}

export function init_menu_bar(app: App, main_window: BrowserWindow) {
	const web_contents = main_window.webContents as WebContents
	const template: MenuItemConstructorOptions[] = [
		{
			label: app.name,
			submenu: [
				{ role: 'about' },
				{ type: 'separator' },
				{
					label: 'Settings...',
					accelerator: 'CmdOrCtrl+,',
					click() {
						web_contents.send('show_settings')
					},
				},
				{ type: 'separator' },
				{ role: 'services' },
				{ type: 'separator' },
				{ role: 'hide' },
				{ role: 'hideOthers' },
				{ role: 'unhide' },
				{ type: 'separator' },
				{ role: 'quit' },
			],
			visible: is.mac,
		},
		{
			label: 'File',
			submenu: [
				{
					label: 'New Playlist',
					accelerator: 'CmdOrCtrl+N',
					click() {
						web_contents.send('newPlaylist', 'root', false)
					},
				},
				{
					label: 'New Playlist Folder',
					click() {
						web_contents.send('newPlaylist', 'root', true)
					},
				},
				{ type: 'separator' },
				{
					label: 'Import...',
					accelerator: 'CmdOrCtrl+O',
					click() {
						web_contents.send('import')
					},
				},
				{ type: 'separator' },
				{
					label: 'Import iTunes Library...',
					click() {
						web_contents.send('itunesImport')
					},
				},
				{ type: 'separator' },
				{
					label: 'Settings...',
					accelerator: 'CmdOrCtrl+,',
					click() {
						web_contents.send('show_settings')
					},
					visible: !is.mac,
				},
				{ type: 'separator', visible: !is.mac },
				is.mac ? { role: 'close' } : { role: 'quit' },
			],
		},
		{
			label: 'Edit',
			submenu: (() => {
				const menu: MenuItemConstructorOptions[] = [
					{ role: 'undo' },
					{ role: 'redo' },
					{ type: 'separator' },
					{ role: 'cut' },
					{ role: 'copy' },
					{ role: 'paste' },
				]
				if (is.mac) {
					menu.push({ role: 'selectAll' })
					menu.push({ type: 'separator' })
				} else {
					menu.push({ type: 'separator' })
					menu.push({ role: 'selectAll' })
				}
				menu.push({
					label: 'Filter',
					accelerator: 'CmdOrCtrl+F',
					click() {
						web_contents.send('filter')
					},
				})
				return menu
			})(),
		},
		{
			label: 'Song',
			submenu: [
				{
					label: 'Play Next',
					accelerator: '',
					click() {
						web_contents.send('selected_tracks_action', 'Play Next')
					},
				},
				{
					label: 'Add to Queue',
					accelerator: '',
					click() {
						web_contents.send('selected_tracks_action', 'Add to Queue')
					},
				},
				{ type: 'separator' },
				{
					label: 'Get Info',
					accelerator: 'CmdOrCtrl+I',
					click() {
						web_contents.send('selected_tracks_action', 'Get Info')
					},
				},
				{ type: 'separator' },
				{
					label: (() => {
						if (is.mac) return 'Reveal in Finder'
						else if (is.windows) return 'Reveal in File Explorer'
						else return 'Reveal in File Manager'
					})(),
					accelerator: 'Shift+CmdOrCtrl+R',
					click() {
						web_contents.send('selected_tracks_action', 'reveal_track_file')
					},
				},
				{ type: 'separator' },
				{
					label: 'Remove from Playlist',
					accelerator: '',
					click() {
						web_contents.send('selected_tracks_action', 'Remove from Playlist')
					},
				},
				{
					label: 'Delete from Library',
					accelerator: 'CmdOrCtrl+Backspace',
					click() {
						web_contents.send('selected_tracks_action', 'Delete from Library')
					},
				},
			],
		},
		{
			label: 'View',
			submenu: [
				{
					label: 'Show Queue',
					id: 'Show Queue',
					type: 'checkbox',
					accelerator: 'CmdOrCtrl+U',
					click() {
						web_contents.send('Show Queue')
					},
				},
				{
					label: 'Show Lyrics',
					id: 'Show Lyrics',
					type: 'checkbox',
					accelerator: 'CmdOrCtrl+L',
					click() {
						web_contents.send('Show Lyrics')
					},
				},
				{
					label: 'Toggle Visualizer',
					type: 'checkbox',
					accelerator: 'CmdOrCtrl+T',
					click() {
						web_contents.send('Toggle Visualizer')
					},
				},
				{
					label: 'Toggle Quick Nav',
					type: 'checkbox',
					accelerator: 'CmdOrCtrl+K',
					click() {
						web_contents.send('ToggleQuickNav')
					},
				},
				{
					label: 'Group Album Tracks',
					type: 'checkbox',
					checked: true,
					click: (item) => {
						web_contents.send('Group Album Tracks', item.checked)
					},
				},
				{ type: 'separator' },
				{ role: 'reload', accelerator: 'CmdOrCtrl+Option+R' },
				{ role: 'forceReload', accelerator: 'Shift+CmdOrCtrl+Option+R' },
				{ role: 'toggleDevTools' },
				{ type: 'separator' },
				{ role: 'resetZoom' },
				{ role: 'zoomIn' },
				{ role: 'zoomOut' },
				{ type: 'separator' },
				{ role: 'togglefullscreen' },
			],
		},
		{
			label: 'Playback',
			submenu: [
				{
					label: 'Pause',
					// no accelerator because it's unreliable
					click() {
						web_contents.send('playPause')
					},
				},
				{
					label: 'Stop',
					// no accelerator because it's unreliable
					click() {
						web_contents.send('Stop')
					},
				},
				{
					label: 'Next',
					accelerator: 'CmdOrCtrl+Right',
					click() {
						web_contents.send('Next')
					},
				},
				{
					label: 'Previous',
					accelerator: 'CmdOrCtrl+Left',
					click() {
						web_contents.send('Previous')
					},
				},
				{ type: 'separator' },
				{
					label: 'Shuffle',
					id: 'Shuffle',
					type: 'checkbox',
					accelerator: 'CmdOrCtrl+S',
					click() {
						web_contents.send('Shuffle')
					},
				},
				{
					label: 'Repeat',
					id: 'Repeat',
					type: 'checkbox',
					accelerator: 'CmdOrCtrl+R',
					click() {
						web_contents.send('Repeat')
					},
				},
				{ type: 'separator' },
				{
					id: 'Volume Up',
					label: 'Volume Up',
					accelerator: 'CmdOrCtrl+Up',
					click() {
						web_contents.send('volumeUp')
					},
				},
				{
					id: 'Volume Down',
					label: 'Volume Down',
					accelerator: 'CmdOrCtrl+Down',
					click() {
						web_contents.send('volumeDown')
					},
				},
			],
		},
		{
			label: 'Window',
			submenu: [
				{
					id: 'Back',
					label: 'Back',
					accelerator: 'CmdOrCtrl+[',
					click() {
						web_contents.send('Back')
					},
				},
				{
					id: 'Forward',
					label: 'Forward',
					accelerator: 'CmdOrCtrl+]',
					click() {
						web_contents.send('Forward')
					},
				},
				{ type: 'separator', visible: is.mac },
				{
					id: 'Select Next List',
					label: 'Select Next List',
					accelerator: 'Ctrl+Tab',
					click() {
						web_contents.send('Select Next List')
					},
				},
				{
					id: 'Select Previous List',
					label: 'Select Previous List',
					accelerator: 'Ctrl+Shift+Tab',
					click() {
						web_contents.send('Select Previous List')
					},
				},
				{ type: 'separator', visible: is.mac },
				{ role: 'minimize' },
				{ role: 'zoom' },
				{ type: 'separator', visible: is.mac },
				{ role: 'front', visible: is.mac },
				{ type: 'separator', visible: is.mac },
				{ role: 'close', visible: !is.mac },
			],
		},
		{
			role: 'help',
			submenu: [
				{
					label: 'Learn More',
					click: async () => {
						await shell.openExternal('https://github.com/kasper9n/ferrum')
					},
				},
			],
		},
	]
	const menu = Menu.buildFromTemplate(template)
	Menu.setApplicationMenu(menu)

	ipc_main.handle('update:Shuffle', (_, checked) => {
		const item = menu.getMenuItemById('Shuffle')
		if (!item) return handle_missing('Shuffle')
		item.checked = checked
	})
	ipc_main.handle('update:Repeat', (_, checked) => {
		const item = menu.getMenuItemById('Repeat')
		if (!item) return handle_missing('Repeat')
		item.checked = checked
	})
	ipc_main.handle('update:Show Queue', (_, checked) => {
		const item = menu.getMenuItemById('Show Queue')
		if (!item) return handle_missing('Show Queue')
		item.checked = checked
	})
	ipc_main.handle('update:Show Lyrics', (_, checked) => {
		const item = menu.getMenuItemById('Show Lyrics')
		if (!item) return handle_missing('Show Lyrics')
		item.checked = checked
	})
}
