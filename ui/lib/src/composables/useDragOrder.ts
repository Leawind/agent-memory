// Drag-and-drop reordering for an ordered list of rows, on SortableJS. Element Plus ships no
// general-purpose sortable list (its own docs reach for SortableJS), and hand-rolled HTML5 drag
// events give no landing preview and no touch support at all. SortableJS leaves a placeholder in the
// gap where the row will land, animates the rows in between, uses the whole row as the drag image,
// and falls back to pointer events on touch screens.
//
// It owns the DOM move; the data move is ours. onEnd reports the two positions SortableJS just
// swapped, and moveRow() applies the same move to the caller's array, which is what re-renders the
// list in the new order. Nothing else about the rows is touched — a drop only reorders.
import Sortable from 'sortablejs'
import { onBeforeUnmount, onMounted, type Ref } from 'vue'

/** Splice-move, the semantics SortableJS reports through onEnd: the dragged row lands at `to` and
 *  the rows in between shift by one, which is what dragging a row past its neighbours looks like.
 *  Out-of-range indices (an aborted drag) leave the list alone. */
export function moveRow<T>(rows: T[], from: number, to: number): void {
  if (from === to || from < 0 || to < 0 || from >= rows.length || to >= rows.length) return
  const [moved] = rows.splice(from, 1)
  rows.splice(to, 0, moved)
}

export interface DragOrderOptions {
  /** Selector of the rows; must match what the caller renders */
  item?: string
  /** Selector of the element a drag may start from — the rows themselves are not drag sources */
  handle?: string
}

/**
 * Make `listEl` sortable and keep `rows` in the order the user drops them into. The list element
 * must be mounted when this composable is set up (call it from `setup` with an always-rendered
 * container).
 */
export function useDragOrder<T>(listEl: Ref<HTMLElement | null>, rows: Ref<T[]>, options: DragOrderOptions = {}) {
  let sortable: Sortable | null = null

  onMounted(() => {
    if (!listEl.value) return
    sortable = Sortable.create(listEl.value, {
      draggable: options.item ?? '.entry',
      handle: options.handle ?? '.entry-handle',
      animation: 150,
      // The row left in the list while another is in flight is the landing preview. Its own class
      // is shared by both drag modes; the floating clone (touch only — a native drag is drawn by the
      // browser from the row itself) gets a separate one so it can be styled as a carried element.
      ghostClass: 'is-ghost',
      fallbackClass: 'is-drag',
      onEnd: (evt) => moveRow(rows.value, evt.oldIndex ?? 0, evt.newIndex ?? 0),
    })
  })

  onBeforeUnmount(() => {
    sortable?.destroy()
    sortable = null
  })
}
