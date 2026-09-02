import { ipc_renderer } from './window'

export function error_popup(err: unknown, crash = false) {
	ipc_renderer.invoke(
		'showMessageBox',
		false,
		{
			type: 'error',
			message: parse_error(err).message,
			detail: error_to_string(err),
		},
		crash,
	)
}

function parse_error(raw_err: unknown): Error {
	let error: Error
	if (raw_err instanceof Error) {
		error = raw_err
	} else {
		error = new Error('Unexpected error: ' + String(raw_err))
	}
	return error
}

export function error_to_string(raw_err: unknown): string {
	const error = parse_error(raw_err)
	const causes: string[] = []

	let cause = error.cause
	while (cause) {
		const cause_error = parse_error(cause)
		causes.push(cause_error.message)
		cause = cause_error.cause
	}

	let result = error.message

	if (causes.length) {
		result += '\n\nCaused by:'

		for (const [i, cause] of causes.entries()) {
			const prefix = causes.length > 1 ? `${i}: ` : ''
			result += `\n    ${prefix}${cause}`
		}
	}

	return result
}

// Crashes on error
export function strict_call<T>(cb: (addon: typeof window.addon) => T): T {
	try {
		const result = cb(window.addon)

		// Handle async errors
		if (result instanceof Promise) {
			return result.catch((raw_err) => {
				const error = parse_error(raw_err)
				error_popup(error, true)
				throw error
			}) as T
		}

		// Handle synchronous result
		return result
	} catch (raw_err) {
		const error = parse_error(raw_err)
		// Handle synchronous errors
		error_popup(error, true)
		throw error
	}
}

type BaseResult<T> = { data: T; error: null } | { data: null; error: Error }
type Result<T> = BaseResult<T> & {
	on_success: (cb: (data: T) => void) => BaseResult<T>
}

// Shows a popup message on error
export function call_sync<T>(cb: (addon: typeof window.addon) => T): Result<T> {
	const result = (() => {
		try {
			const data = cb(window.addon)

			const result = { data, error: null }
			return result
		} catch (raw_err) {
			const error = parse_error(raw_err)
			error_popup(error)

			const result = { data: null, error }
			return result
		}
	})() as Result<T>
	result.on_success = (cb) => {
		if (result.error === null) {
			cb(result.data)
		}
		return result
	}
	return result
}
