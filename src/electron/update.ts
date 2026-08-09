import { app, dialog } from 'electron'
import package_json from '../../package.json'

export type UpdateJson = {
	[channel: string]: {
		url: string
		version: string
	}
}

export type JsonValue =
	string | number | boolean | null | JsonValue[] | { [key: string]: JsonValue }

function popup(message: string, detail: string) {
	console.error(`${message}: ${detail}`)
	dialog.showMessageBox({
		type: 'error',
		message,
		detail,
	})
	return null
}

async function fetch_json(url: string) {
	const response = await fetch(url, {}).catch((error: Error) => error)

	if (response instanceof Error) {
		return { error: response.message, data: null }
	} else if (response.status !== 200) {
		return { error: `${response.status}: ${response.statusText}\n\n${url}`, data: null }
	}

	const value: JsonValue = await response.json().catch(() => {
		return null
	})
	if (value === null) {
		// also handle JSON null value as error
		return { error: 'Could not parse JSON', data: null }
	}
	return { data: value, error: null }
}

export async function check_for_updates() {
	const release_result = await fetch_json(package_json['latest-release-api-url'])
	if (release_result.error) {
		return popup('Failed to check for updates', release_result.error)
	}
	const release = release_result.data

	if (
		!release ||
		typeof release !== 'object' ||
		!('tag_name' in release) ||
		typeof release.tag_name !== 'string'
	) {
		return popup('Failed to check for updates', 'Could not parse JSON')
	}

	const update_json_url = `${package_json.repository}/releases/download/${release.tag_name}/update.json`

	const update_result = await fetch_json(update_json_url)
	if (update_result.error) {
		return popup('Failed to check for updates', update_result.error)
	}
	const update_json = update_result.data as UpdateJson
	console.log(update_json)

	const channel = update_json['stable-0']
	if (!channel) {
		return popup('Failed to check for updates', 'Could not find update channel')
	}

	return {
		channel,
		app_version: app.getVersion(),
		body: typeof release.body === 'string' ? release.body : 'Changelog not available',
	}
}
