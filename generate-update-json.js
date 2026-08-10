import fs from 'fs'
import path from 'path'

const dirname = import.meta.dirname

const package_json = JSON.parse(fs.readFileSync(path.join(dirname, 'package.json')))

/** @type {import('./src/electron/update').UpdateJson} */
const update_json = {
	// In the future we can create a stable-1 channel, to avoid notifying users about versions they shouldn't upgrade to. For example if their OS becomes unsupported, or If we decide to drop version migration logic.
	'stable-0': {
		url: package_json.updates_url_prefix + '/v' + package_json.version,
		version: package_json.version,
	},
}

console.log('Generating update.json: \n%o', update_json)

const build_dir = path.join(dirname, 'build')
fs.mkdirSync(build_dir, { recursive: true })
fs.writeFileSync(path.join(build_dir, 'update.json'), JSON.stringify(update_json, null, '\t'))
