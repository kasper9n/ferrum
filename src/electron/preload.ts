import path from 'path'
import is from './is'
import { createRequire } from 'node:module'
const require = createRequire(import.meta.url)
const addon = require('ferrum-addon')
import { webUtils } from 'electron'

window.addon = addon
window.is_dev = is.dev
window.local_data_path = process.env.LOCAL_DATA ? path.resolve(process.env.LOCAL_DATA) : undefined
window.library_path = process.env.LIBRARY ? path.resolve(process.env.LIBRARY) : undefined
window.is_mac = is.mac
window.is_windows = is.windows

window.get_file_path = (file) => {
	const path = webUtils.getPathForFile(file)
	return path
}
