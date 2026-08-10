import { type Updater, type Writable, writable } from 'svelte/store'
import type { TrackListDetails } from 'ferrum-addon'

export function get_duration(dur: number) {
	dur = Math.round(dur)
	const secs = dur % 60
	let secs_text = String(secs)
	if (secs < 10) secs_text = '0' + secs
	const mins = (dur - secs) / 60
	return mins + ':' + secs_text
}

let time_formatter: Intl.DateTimeFormat | undefined

export function format_date(timestamp: number) {
	const date = new Date(timestamp)
	const month = date.getMonth() + 1
	const month_text = (month < 10 ? '0' : '') + month
	const day = date.getDate()
	const day_text = (day < 10 ? '0' : '') + day
	if (!time_formatter) {
		time_formatter = new Intl.DateTimeFormat(undefined, {
			hour: '2-digit',
			minute: '2-digit',
		})
	}
	const clock = time_formatter.format(date)
	return date.getFullYear() + '/' + month_text + '/' + day_text + ' ' + clock
}

type ShortcutOptions = {
	shift?: boolean
	alt?: boolean
	cmd_or_ctrl?: boolean
}
const is_mac = navigator.userAgent.indexOf('Mac') !== -1

export function check_modifiers(e: KeyboardEvent | MouseEvent, options: ShortcutOptions) {
	const target = {
		shift: options.shift || false,
		alt: options.alt || false,
		ctrl: (!is_mac && options.cmd_or_ctrl) || false,
		meta: (is_mac && options.cmd_or_ctrl) || false,
	}

	const pressed = {
		shift: !!e.shiftKey,
		alt: !!e.altKey,
		ctrl: !!e.ctrlKey,
		meta: !!e.metaKey,
	}

	const ignore_ctrl = is_mac && e instanceof MouseEvent

	return (
		pressed.shift === target.shift &&
		pressed.alt === target.alt &&
		(pressed.ctrl === target.ctrl || ignore_ctrl) &&
		pressed.meta === target.meta
	)
}

export function check_shortcut(e: KeyboardEvent, key: string, options: ShortcutOptions = {}) {
	if (e.key.toUpperCase() !== key.toUpperCase()) return false
	return check_modifiers(e, options)
}
export function check_mouse_shortcut(e: MouseEvent, options: ShortcutOptions = {}) {
	return check_modifiers(e, options)
}

export function clamp_number(min: number, max: number, value: number) {
	if (value < min) return min
	if (value > max) return max
	return value
}

type FlattenedListMenuItem = { label: string; enabled: boolean; id: string }
export function flatten_child_lists(
	track_list: TrackListDetails,
	track_lists: Record<string, TrackListDetails>,
	indent_prefix: string,
) {
	let flat: FlattenedListMenuItem[] = []
	for (const child_list_id of track_list.children || []) {
		const child_list = track_lists[child_list_id]
		if (child_list.kind === 'folder') {
			const child_flat = flatten_child_lists(child_list, track_lists, indent_prefix + '   ')
			flat.push({
				label: indent_prefix + child_list.name,
				enabled: false,
				id: child_list.id,
			})
			flat = flat.concat(child_flat)
		} else if (child_list.kind === 'playlist') {
			flat.push({
				label: indent_prefix + child_list.name,
				enabled: true,
				id: child_list.id,
			})
		}
	}
	return flat
}

type GetterWritable<T> = Writable<T> & { get(): T }
export function getter_writable<T>(value: T): GetterWritable<T> {
	const { subscribe, set } = writable(value)
	return {
		subscribe: subscribe,
		set(new_value: T) {
			value = new_value
			set(new_value)
		},
		update(updater: Updater<T>) {
			value = updater(value)
			set(value)
		},
		get: () => value,
	}
}

export function assert_unreachable(x: never) {
	console.error('Unreachable reached', x)
}

/** A requestAnimationFrame instance that prevents multiple simultaneous calls */
export function create_singular_request_animation_frame() {
	let scheduled: number | undefined

	return (callback: FrameRequestCallback): number => {
		if (scheduled) return scheduled

		scheduled = requestAnimationFrame((time) => {
			scheduled = undefined
			callback(time)
		})
		return scheduled
	}
}

export function prevent_default(e: Event) {
	e.preventDefault()
}

export function prevent_default_self(e: Event) {
	if (e.target === e.currentTarget) {
		e.preventDefault()
	}
}
