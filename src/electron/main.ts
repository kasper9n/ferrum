import { app, ipcMain, session, BrowserWindow, dialog, protocol, net } from 'electron'
import is from './is'
import addon from '../../ferrum-addon'

if (is.dev) app.setName('Ferrum Dev')

import { init_menu_bar } from './menubar'
import('./ipc')
import path from 'path'
import url from 'url'
import { ipc_main } from './typed_ipc'

async function close_cache_db() {
	await addon.close_cache_db()
}
function get_logs_dir() {
	try {
		return addon.get_logs_dir()
	} catch (_) {
		return null
	}
}
const logs_dir = get_logs_dir()

let triggering_crash = false
export function trigger_crash(popup: { msg: string; error: Error | string } | null) {
	if (triggering_crash) {
		return
	}
	triggering_crash = true
	quitting = true
	const close_promise = close_cache_db()
	app.whenReady().then(async () => {
		if (popup) {
			dialog.showMessageBoxSync({
				type: 'error',
				message: popup.msg,
				detail: popup.error instanceof Error ? popup.error.stack : popup.error,
				title: 'Error',
			})
		}
		browser_windows.main_window?.close()
		await close_promise
		app.quit()
		setTimeout(() => {
			console.log('Exiting with code 1')
			app.exit(1)
		})
	})
}
process.on('uncaughtException', (error) => {
	trigger_crash({ msg: 'Unhandled Error', error })
})
process.on('unhandledRejection', (error: Error) => {
	trigger_crash({ msg: 'Unhandled Promise Rejection', error })
})

const app_data = app.getPath('appData')
const local_data_path = process.env.LOCAL_DATA ? path.resolve(process.env.LOCAL_DATA) : null
if (is.dev) {
	const data_path =
		local_data_path ??
		path.join(__dirname, '../../src-native/appdata/LocalData/space.kasper.ferrum')
	const electron_data_path = path.join(data_path, 'Electron Data')
	app.setPath('userData', electron_data_path)
} else {
	const data_path = local_data_path ?? path.join(app_data, 'space.kasper.ferrum')
	const electron_data_path = path.join(data_path, 'Electron Data')
	app.setPath('userData', electron_data_path)
}

let quitting = false
let app_loaded = false

app.on('window-all-closed', async () => {
	if (!triggering_crash) {
		await close_cache_db()
		app.quit()
	}
})

protocol.registerSchemesAsPrivileged([
	{
		scheme: 'app',
		privileges: {
			standard: true, // standard URLs have origins, which is required for history.pushState
			supportFetchAPI: true,
		},
	},
	{
		scheme: 'track',
		privileges: {
			standard: true,
			stream: true,
			supportFetchAPI: true,
		},
	},
])

export const browser_windows = {
	main_window: null as BrowserWindow | null,
}

app.whenReady().then(async () => {
	let main_window: BrowserWindow | null = new BrowserWindow({
		width: 1305,
		height: 1000,
		minWidth: 850,
		minHeight: 400,
		titleBarStyle: is.mac ? 'hidden' : 'default',
		webPreferences: {
			contextIsolation: false,
			nodeIntegration: true,
			preload: path.resolve(__dirname, './preload.js'),
		},
		backgroundColor: '#0D1115',
		show: false,
	})
	browser_windows.main_window = main_window

	// Deprecated, but Range requests do not work: https://github.com/electron/electron/issues/38749
	// Possible workaround: https://github.com/laurent22/joplin/blob/2654b33620775080d1d59c552259d41e33dad3d2/packages/app-desktop/utils/customProtocols/handleCustomProtocols.ts
	protocol.registerFileProtocol('track', (request, callback) => {
		const url = decodeURI(request.url)
		const path = url.substring(6)
		callback(path)
	})

	session.defaultSession.webRequest.onHeadersReceived((details, callback) => {
		callback({
			responseHeaders: {
				...details.responseHeaders,
				'Content-Security-Policy': ["script-src 'self' 'unsafe-inline';"],
			},
		})
	})

	const vite_dev_server_url = process.env.VITE_DEV_SERVER_URL ?? ''
	if (is.dev && !vite_dev_server_url) {
		throw new Error('VITE_DEV_SERVER_URL missing')
	}

	const web_folder = path.join(path.dirname(__dirname), 'web')
	const origin = 'app://app'

	protocol.handle('app', (request) => {
		const req_url = new URL(request.url)
		if (req_url.hostname === 'trackimg') {
			return new Promise((resolve) => {
				const track_path = decodeURIComponent(req_url.searchParams.get('path') ?? '')
				const cache_db_path = decodeURIComponent(req_url.searchParams.get('cache_db_path') ?? '')

				if (quitting) {
					// If we're quitting, don't call async addon functions. It seems to cause panics.
					return new Response('Currently quitting', { status: 500 })
				}
				addon
					.read_small_cover_async(track_path, 0, cache_db_path)
					.then((buffer) => {
						if (buffer === null) {
							resolve(new Response(null, { status: 404 }))
						} else {
							resolve(new Response(Buffer.from(buffer)))
						}
					})
					.catch((error) => {
						resolve(new Response(error, { status: 500 }))
						console.error(`Could not read cover "${track_path}":`, error)
					})
			})
		} else if (req_url.hostname === 'app') {
			const accepts_html =
				request.headers
					.get('accept')
					?.split(',')
					.map((mime_type) => mime_type.trim())
					.includes('text/html') ?? false

			if (request.method === 'GET' && accepts_html) {
				const html_path = url.pathToFileURL(path.join(web_folder, 'index.html')).toString()
				return net.fetch(html_path)
			}

			const file_path = path.join(web_folder, req_url.pathname)
			const file_url = url.pathToFileURL(file_path).toString()
			return net.fetch(file_url)
		} else {
			return new Response('Unknown protocol', { status: 400 })
		}
	})

	if (is.dev) {
		main_window.loadURL(new URL(vite_dev_server_url).origin + '/playlist/root')
	} else {
		main_window.loadURL(`${origin}/playlist/root`)
	}

	if (is.dev) main_window.webContents.openDevTools()

	main_window.once('ready-to-show', () => {
		main_window?.show()
	})

	main_window.on('close', (e) => {
		if (!quitting && is.mac) {
			e.preventDefault()
			app.hide()
		}
	})
	main_window.on('closed', () => {
		main_window = null
		browser_windows.main_window = main_window
	})
	main_window.webContents.on('render-process-gone', (_e, details) => {
		if (
			details.reason !== 'clean-exit' &&
			details.reason !== 'abnormal-exit' &&
			details.reason !== 'killed'
		) {
			trigger_crash({
				msg: `Crashed with code ${details.exitCode} (${details.reason})`,
				error: `Error message was logged to ${logs_dir}`,
			})
		}
	})
	ipc_main.handle('app_loaded', () => {
		app_loaded = true
	})

	// doesn't always fire on Windows :(
	let gonna_quit = false
	app.on('before-quit', (e) => {
		if (quitting) {
			return
		} else if (app_loaded) {
			if (gonna_quit) {
				console.log('Already preparing to quit')
				return
			}
			console.log('Preparing to quit')
			e.preventDefault()
			main_window?.webContents.send('gonnaQuit')
			gonna_quit = true
			ipcMain.once('readyToQuit', () => {
				console.log('Quitting gracefully')
				quitting = true
				main_window?.close()
			})
		} else {
			console.log('Quitting immediately')
			quitting = true
			main_window?.close()
		}
	})

	app.on('activate', () => {
		if (main_window !== null) {
			main_window.show()
		}
	})

	init_menu_bar(app, main_window)
})

process.on('SIGINT', () => {
	console.log('Trying to quit')
	app.quit()
})
