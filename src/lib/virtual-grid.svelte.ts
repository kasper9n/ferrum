import { untrack } from 'svelte'

export type Column = {
	name: string
	key: string
	width: number
	is_pct?: boolean
	offset?: number
	/** Handles both creation and updating of rows */
	cell_render?: (element: HTMLElement, value: unknown) => void
}

export const RefreshLevel = {
	Nothing: 0,
	// Render rows that go in/out of the viewport
	NewRows: 1,
	// Render all rows in the viewport
	AllRows: 2,
} as const
export type RefreshLevel = (typeof RefreshLevel)[keyof typeof RefreshLevel]

type Row = {
	/* null means it will render */
	element: HTMLElement | null
	/* null means it will be removed */
	index: number | null
	rendered: boolean
}

/**
 * Note that parentElement is not reactive.
 * Do not add other elements into the row elements. Things would break because cells are referenced by indexing into the row's children.
 */
export class VirtualGrid<I, R extends Record<string, unknown>> {
	main_element?: HTMLElement
	viewport?: HTMLElement
	size_observer?: ResizeObserver
	refreshing: RefreshLevel = RefreshLevel.Nothing

	private constructor(
		public source_items: I[],
		public options: {
			buffer?: number
			row_prepare: (source_items: I, index: number) => R
			row_render?: (element: HTMLElement, item: R, index: number) => void
		},
	) {}

	static create<I, R extends Record<string, unknown>>(
		source_items: I[],
		options: {
			buffer?: number
			row_prepare: (source_items: I, index: number) => R
			row_render?: (element: HTMLElement, item: R, index: number) => void
		},
	) {
		return new VirtualGrid<I, R>(source_items, options)
	}

	row_height = 24

	set_source_items(source_items: I[]) {
		this.source_items = source_items
		this.#update_viewport_size()
		this.refresh(RefreshLevel.AllRows)
	}

	viewport_row_count = 0

	#update_viewport_size() {
		const viewport_height = this.viewport?.clientHeight ?? 0
		this.viewport_row_count = Math.ceil(viewport_height / this.row_height)

		const height = this.source_items.length * this.row_height
		if (this.main_element) {
			this.main_element.style.height = height + 'px'
		}
	}

	rows: Row[] = []

	#recalculate_visible_indexes() {
		if (!this.viewport) {
			return
		}
		const buffer = this.options.buffer ?? 5
		const rendered_count = this.viewport_row_count + buffer * 2

		let start_index = Math.max(0, Math.floor(this.viewport.scrollTop / this.row_height - buffer))
		const end_index = Math.min(this.source_items.length - 1, start_index - 1 + rendered_count)
		if (end_index - start_index + 1 < rendered_count) {
			// fill backwards when scrolled to the end
			start_index = Math.max(0, end_index + 1 - rendered_count)
		}

		// figure out which indexes should now be added
		const new_visible_indexes: number[] = []
		for (let i = start_index; i <= end_index; i++) {
			const exists = this.rows.find((row) => row.index === i)
			if (!exists) {
				new_visible_indexes.push(i)
			}
		}

		// update the visible indexes
		for (let i = 0; i < this.rows.length; i++) {
			const row = this.rows[i]
			const still_visible = row.index !== null && row.index >= start_index && row.index <= end_index
			if (!still_visible) {
				const new_index = new_visible_indexes.pop()
				if (new_index !== undefined) {
					// update it to a new visible index
					row.index = new_index
					row.rendered = false
				} else {
					// if there are no new visible indexes left, remove it
					row.index = null
					row.rendered = false
				}
			}
		}
		if (new_visible_indexes.length > 0) {
			// add new visible indexes
			for (const index of new_visible_indexes) {
				this.rows.push({
					element: null,
					index,
					rendered: false,
				})
			}
		}
	}

	refresh(level: RefreshLevel = RefreshLevel.AllRows) {
		if (this.refreshing) {
			if (level > this.refreshing) {
				this.refreshing = level
			}
			return
		}
		this.refreshing = level

		requestAnimationFrame(() => {
			this.#recalculate_visible_indexes()
			if (this.refreshing === RefreshLevel.AllRows) {
				for (const row of this.rows) {
					row.rendered = false
				}
			}
			this.#render()
			this.refreshing = 0
		})
	}

	columns: Column[] = $state([])
	set_columns(columns: Column[]) {
		const old_columns = untrack(() => this.columns)
		const total_fixed_width = columns.reduce((sum, col) => sum + (col.is_pct ? 0 : col.width), 0)
		const total_percent_pct = columns.reduce((sum, col) => sum + (col.is_pct ? col.width : 0), 0)
		const container_width = this.viewport?.clientWidth ?? total_fixed_width
		const total_percent_width = container_width - total_fixed_width
		let offset = 0
		const new_columns = columns.map((col) => {
			col = { ...col }
			if (col.is_pct) {
				const pct = col.width / total_percent_pct
				col.width = pct * total_percent_width
			}
			col.offset = offset
			offset += col.width
			return col
		})

		let resize_only = true
		if (old_columns.length !== new_columns.length) {
			resize_only = false
		}
		for (let i = 0; i < old_columns.length; i++) {
			if (new_columns[i]?.key !== old_columns[i]?.key) {
				resize_only = false
				break
			}
		}

		this.columns = new_columns
		if (resize_only) {
			for (const row of this.rows) {
				if (!row.element) {
					throw new Error('Unexpected missing row element')
				}
				for (let ci = 0; ci < new_columns.length; ci++) {
					const column = new_columns[ci]
					const cell = row.element.children[ci] as HTMLElement
					cell.style.width = `${column.width}px`
					cell.style.translate = `${column.offset}px 0`
				}
			}
		} else {
			// make all rows fully rerender
			for (const row of this.rows) {
				row.element?.remove()
			}
			this.rows = []

			this.refresh(RefreshLevel.NewRows)
		}
		return new_columns
	}

	#render() {
		// prepare rows, before any DOM updates
		const items: R[] = []
		for (const row of this.rows) {
			if (row.index === null || row.rendered) {
				continue
			}
			const row_item = this.options.row_prepare(this.source_items[row.index], row.index)
			items[row.index] = row_item
		}

		// create new row elements
		for (const row of this.rows) {
			if (row.element || row.index === null) {
				continue
			}
			const row_element = document.createElement('div')
			row_element.className = 'row'
			row_element.setAttribute('role', 'row')
			row.element = row_element
			this.main_element?.appendChild(row_element)

			for (const column of this.columns) {
				const cell = document.createElement('div')
				cell.className = `cell ${column.key}`
				cell.style.width = `${column.width}px`
				cell.style.translate = `${column.offset}px 0`
				row_element.appendChild(cell)
			}
		}

		// render rows
		for (const row of this.rows) {
			if (row.rendered || row.index === null) {
				continue
			}
			if (!row.element) {
				throw new Error('Unexpected missing row element')
			}
			row.element.style.translate = `0 ${row.index * this.row_height}px`
			row.element.setAttribute('aria-rowindex', String(row.index + 1))
			const row_item = items[row.index]
			for (let ci = 0; ci < this.columns.length; ci++) {
				const column = this.columns[ci]
				const cell = row.element.children[ci] as HTMLElement
				let cell_value = row_item[column.key]
				if (cell_value === undefined || cell_value === null) {
					cell_value = ''
				}
				if (column.cell_render) {
					column.cell_render(cell, cell_value)
				} else {
					cell.textContent = String(cell_value)
				}
			}
			this.options.row_render?.(row.element, row_item, row.index)
			row.rendered = true
		}

		// delete rows that are no longer visible
		for (let i = this.rows.length - 1; i >= 0; i--) {
			const row = this.rows[i]
			if (row.index === null) {
				row.element?.remove()
				this.rows.splice(i, 1)
			}
		}
	}

	scroll_to_index(index: number, scroll_margin_bottom = 0) {
		if (!this.viewport) {
			throw new Error('No viewport')
		}
		const dummy = document.createElement('div')
		dummy.style.height = this.row_height + 'px'
		dummy.style.position = 'absolute'
		dummy.style.top = index * this.row_height + 'px'
		dummy.style.scrollMarginBottom = scroll_margin_bottom + 'px'
		this.viewport.prepend(dummy)
		dummy.scrollIntoView({ behavior: 'instant', block: 'nearest' })
		dummy.remove()
	}

	setup(node: HTMLElement) {
		this.main_element = node

		const viewport_result = this.main_element.parentElement
		if (!viewport_result) {
			throw new Error('No viewport')
		}
		const viewport = viewport_result
		this.viewport = viewport

		this.#update_viewport_size()

		this.size_observer = new ResizeObserver(() => {
			this.set_columns(this.columns)
			this.#update_viewport_size()
			this.refresh(RefreshLevel.NewRows)
		})
		this.size_observer.observe(this.viewport)

		const on_scroll = () => this.refresh(RefreshLevel.NewRows)
		const on_keydown = (e: KeyboardEvent) => {
			let prevent = true
			if (e.key === 'Home') viewport.scrollTop = 0
			else if (e.key === 'End') viewport.scrollTop = viewport.scrollHeight
			else if (e.key === 'PageUp') viewport.scrollTop -= viewport.clientHeight
			else if (e.key === 'PageDown') viewport.scrollTop += viewport.clientHeight
			else prevent = false
			if (prevent) e.preventDefault()
		}
		viewport.addEventListener('scroll', on_scroll)
		viewport.addEventListener('keydown', on_keydown)
		return () => {
			viewport.removeEventListener('scroll', on_scroll)
			viewport.removeEventListener('keydown', on_keydown)
			this.size_observer?.disconnect()
		}
	}
	attach() {
		// This is a function in order to make `this` work
		return (node: HTMLElement) => this.setup(node)
	}
}
