import type { TrackID } from 'ferrum-addon'
import { get, writable } from 'svelte/store'
import { getter_writable } from './helpers'
import { ipc_renderer } from './window'
import { track_exists } from '$lib/data.svelte'
import quit from './quit'

export const queue_visible = writable(false)
export function toggle_queue_visibility() {
	queue_visible.update((v) => !v)
}

export type QueueItem = {
	qId: number
	id: TrackID
	non_shuffle_pos?: number
}
let last_qid = -1
function new_queue_item(id: TrackID): QueueItem {
	last_qid++
	return {
		qId: last_qid,
		id,
	}
}

/**
 * If `hasCurrent` is true, the current track is past[length-1]
 */
export type Queue = {
	past: QueueItem[]
	current: {
		item: QueueItem
		from_auto_queue: boolean
	} | null
	user_queue: QueueItem[]
	auto_queue: QueueItem[]
}
export const queue = (() => {
	const store = getter_writable({
		past: [],
		user_queue: [],
		current: null,
		auto_queue: [],
	} as Queue)
	return {
		subscribe: store.subscribe,
		set: store.set,
		update: store.update,
		get: store.get,
		getCurrent: get_current,
		getQueueLength: get_queue_length,
		getByQueueIndex: get_by_queue_index,
		prependToUserQueue: prepend_to_user_queue,
		appendToUserQueue: append_to_user_queue,
		removeIndexes: remove_indexes,
		removeDeleted: remove_deleted,
	}
})()

let queue_file_path = ''
let queue_loaded = false
const MAX_HISTORY = 1000
const SAVE_DEBOUNCE_MS = 1000
let pending_save_timeout: ReturnType<typeof setTimeout> | null = null

function limit_history(past: QueueItem[]) {
	if (past.length <= MAX_HISTORY) return past
	return past.slice(past.length - MAX_HISTORY)
}

function save_queue_state_now() {
	if (!queue_loaded) return
	return window.addon
		.save_queue_state(
			{
				past: queue.get().past,
				current: queue.get().current ?? undefined,
				user_queue: queue.get().user_queue,
				auto_queue: queue.get().auto_queue,
				last_qid,
				shuffle: get(shuffle),
				repeat: repeat.get(),
			},
			queue_file_path,
		)
		.catch(() => {})
}
function save_queue_state() {
	if (!queue_loaded) return
	if (pending_save_timeout) {
		clearTimeout(pending_save_timeout)
	}
	pending_save_timeout = setTimeout(() => {
		pending_save_timeout = null
		save_queue_state_now()
	}, SAVE_DEBOUNCE_MS)
}

quit.set_handler('queue-state', async () => {
	if (pending_save_timeout) {
		clearTimeout(pending_save_timeout)
		pending_save_timeout = null
	}
	await save_queue_state_now()
})

export function init_queue_persistence(file_path: string) {
	if (queue_loaded) return
	queue_file_path = file_path

	const saved = window.addon.load_queue_state(queue_file_path)
	if (saved) {
		const past = saved.past.filter((item) => track_exists(item.id))
		const current = saved.current && track_exists(saved.current.item.id) ? saved.current : null
		const user_queue = saved.user_queue.filter((item) => track_exists(item.id))
		const auto_queue = saved.auto_queue.filter((item) => track_exists(item.id))
		if (current) {
			past.push(current.item)
		}
		const limited_past = limit_history(past)

		last_qid = Math.max(
			-1,
			saved.last_qid,
			...limited_past.map((item) => item.qId),
			...user_queue.map((item) => item.qId),
			...auto_queue.map((item) => item.qId),
		)
		repeat.set(saved.repeat)
		shuffle.set(saved.shuffle)
		queue.set({ past: limited_past, current: null, user_queue, auto_queue })
	}

	queue.subscribe(save_queue_state)
	queue_loaded = true
}

/** Fisher-Yates shuffle */
function shuffle_array<T>(array: Array<T>) {
	let current_index = array.length,
		random_index

	// While there remain elements to shuffle.
	while (current_index !== 0) {
		// Pick a remaining element.
		random_index = Math.floor(Math.random() * current_index)
		current_index--

		// And swap it with the current element.
		;[array[current_index], array[random_index]] = [array[random_index], array[current_index]]
	}

	return array
}

export const shuffle = writable(false)
function apply_shuffle(shuffle: boolean) {
	queue.update((q) => {
		if (shuffle) {
			for (let i = 0; i < q.auto_queue.length; i++) {
				q.auto_queue[i].non_shuffle_pos = i
			}
			q.auto_queue = shuffle_array(q.auto_queue)
			return q
		} else {
			const old_items: Partial<QueueItem[]> = []
			const new_items: QueueItem[] = []
			for (const item of q.auto_queue) {
				if (item.non_shuffle_pos !== undefined) {
					old_items[item.non_shuffle_pos] = item
				} else {
					new_items.push(item)
				}
			}
			const old_items_clean = old_items.filter((i) => i !== undefined) as QueueItem[]
			q.auto_queue = new_items.concat(old_items_clean)
			return q
		}
	})
}
shuffle.subscribe(($shuffle) => {
	apply_shuffle($shuffle)
	ipc_renderer.invoke('update:Shuffle', $shuffle)
	save_queue_state()
})
ipc_renderer.on('Shuffle', () => {
	shuffle.update((value) => !value)
})

export const repeat = getter_writable(false)
ipc_renderer.on('Repeat', () => {
	repeat.update((value) => !value)
})
repeat.subscribe(($repeat) => {
	ipc_renderer.invoke('update:Repeat', $repeat)
	save_queue_state()
})

export function get_current() {
	return queue.get().current?.item ?? null
}
export function get_queue_length() {
	const { past, current, user_queue, auto_queue } = queue.get()
	return past.length + Number(!!current) + user_queue.length + auto_queue.length
}
export function get_by_queue_index(index: number) {
	const { past, current, user_queue, auto_queue } = queue.get()
	if (index < past.length) {
		return past[index]
	}
	index -= past.length
	if (current && index === 0) {
		return current.item
	}
	index -= Number(!!current)
	if (index < user_queue.length) {
		return user_queue[index]
	}
	index -= user_queue.length
	return auto_queue[index]
}

export function prepend_to_user_queue(track_ids: TrackID[]) {
	const items = track_ids.map(new_queue_item)
	queue.update((q) => {
		q.user_queue.unshift(...items)
		return q
	})
}
export function append_to_user_queue(track_ids: TrackID[]) {
	const items = track_ids.map(new_queue_item)
	queue.update((q) => {
		q.user_queue.push(...items)
		return q
	})
}

export function clear_user_queue() {
	queue.update((q) => {
		q.user_queue = []
		return q
	})
}

export function move_indexes(indexes: number[], new_index: number, to_user_queue: boolean) {
	const items: QueueItem[] = []
	queue.update((q) => {
		// Sort descending. We need to remove the last indexes first to not mess up the indexes
		for (const index of indexes.sort((a, b) => b - a)) {
			const removed_item = remove_index(q, index)
			if (removed_item) {
				items.push(removed_item)
				if (index < new_index) {
					new_index--
				}
			} else {
				items.push(new_queue_item(get_by_queue_index(index).id))
			}
		}
		return q
	})
	// We sorted the indexes descending, so now reverse them
	return insert_items(items.reverse(), new_index, to_user_queue)
}

function insert_items(items: QueueItem[], index: number, to_user_queue: boolean) {
	queue.update((q) => {
		const user_queue_index = q.past.length + Number(!!q.current)
		index -= user_queue_index

		if (index < q.user_queue.length || to_user_queue) {
			q.user_queue.splice(index, 0, ...items)
			return q
		}
		index -= q.user_queue.length
		q.auto_queue.splice(index, 0, ...items)
		return q
	})
	return {
		from: index,
		to: index + items.length,
	}
}

export function insert_ids(ids: TrackID[], index: number, bias_top = false) {
	return insert_items(ids.map(new_queue_item), index, bias_top)
}

function remove_index(q: Queue, index: number): QueueItem | null {
	const up_next_index = q.past.length + Number(!!q.current)
	if (index < up_next_index) {
		return null
	}
	index -= up_next_index
	if (index < q.user_queue.length) {
		const [removed] = q.user_queue.splice(index, 1)
		return removed
	}
	index -= q.user_queue.length
	const [removed] = q.auto_queue.splice(index, 1)
	return removed
}

export function remove_indexes(indexes: number[]) {
	queue.update((q) => {
		// we need to remove the last indexes first to not mess up the indexes
		for (const index of indexes.sort((a, b) => a - b).reverse()) {
			remove_index(q, index)
		}
		return q
	})
}
export function remove_deleted() {
	const q = queue.get()
	const past = q.past.filter((qi) => track_exists(qi.id))
	const current = q.current && track_exists(q.current.item.id) ? q.current : null
	const user_queue = q.user_queue.filter((qi) => track_exists(qi.id))
	const auto_queue = q.auto_queue.filter((qi) => track_exists(qi.id))
	if (
		past.length !== q.past.length ||
		current !== null ||
		user_queue.length !== q.user_queue.length ||
		auto_queue.length !== q.auto_queue.length
	) {
		queue.set({ past, current, user_queue, auto_queue })
	}
}

export function next() {
	const q = queue.get()
	if (q.current) {
		q.past.push(q.current.item)
		q.past = limit_history(q.past)
	}
	if (q.user_queue.length) {
		q.current = {
			item: q.user_queue.shift()!,
			from_auto_queue: false,
		}
	} else if (q.auto_queue.length) {
		if (repeat.get() && q.current) {
			q.auto_queue.push(new_queue_item(q.current.item.id))
		}
		q.current = {
			item: q.auto_queue.shift()!,
			from_auto_queue: true,
		}
	} else {
		q.current = null
	}
	queue.set(q)
}
export function prev() {
	const q = queue.get()
	if (q.past.length) {
		if (q.current) {
			q.user_queue.unshift(q.current.item)
		}
		q.current = {
			item: q.past.pop()!,
			from_auto_queue: false,
		}
		queue.set(q)
	}
}

export function set_new_queue(new_ids: TrackID[], new_current_index: number) {
	const auto_queue = new_ids.splice(new_current_index + 1)
	const current = new_ids.pop()
	queue.update((q) => {
		if (q.current) {
			q.past.push(q.current.item)
			q.past = limit_history(q.past)
		}
		q.current = current
			? {
					item: new_queue_item(current),
					from_auto_queue: true,
				}
			: null
		q.auto_queue = auto_queue.map(new_queue_item)
		return q
	})
	queue.removeDeleted()
	apply_shuffle(get(shuffle))
}
