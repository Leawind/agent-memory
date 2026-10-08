// Drag-and-drop reordering for a plain array of rows, on native HTML5 drag events (no extra
// dependency). The dragged index lives in a ref instead of dataTransfer: the handlers stay pure
// functions that tests can drive directly, and browsers that restrict dataTransfer on dragover
// still work. Wire-up: `@dragstart` on the handle, `@dragover.prevent` + `@drop` on the row.
import { ref, type Ref } from 'vue'

export function useDragOrder<T>(rows: Ref<T[]>) {
  const dragIndex = ref<number | null>(null)
  /** The row the pointer currently hovers (highlight target while dragging) */
  const overIndex = ref<number | null>(null)

  function onDragStart(i: number): void {
    dragIndex.value = i
    overIndex.value = i
  }

  /** Hover highlight only: the move itself happens on drop, so dragging across the list never
   *  mutates the rows the pointer passes over */
  function onDragOver(i: number): void {
    if (dragIndex.value === null) return
    overIndex.value = i
  }

  /** Moving a row is a splice, not a swap: the dragged row lands where it was dropped and the
   *  displaced rows shift by one, which is what dragging past a neighbour looks like. */
  function onDrop(i: number): void {
    const from = dragIndex.value
    dragIndex.value = null
    overIndex.value = null
    if (from === null || from === i) return
    const [moved] = rows.value.splice(from, 1)
    rows.value.splice(i, 0, moved)
  }

  function onDragEnd(): void {
    dragIndex.value = null
    overIndex.value = null
  }

  return { dragIndex, overIndex, onDragStart, onDragOver, onDrop, onDragEnd }
}
