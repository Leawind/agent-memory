// Edge resizing for the floating editor dialogs: drag strips sit on both the left and the
// right dialog edge. The dialog stays centered while resizing — both edges move outward
// (or inward) symmetrically, so the width changes by twice the pointer delta and each edge
// tracks the pointer 1:1. The upper bound is the viewport minus a fixed margin (no fixed
// pixel cap); the strips render in the dialogs' header slots (absolutely positioned
// against the dialog root); the scrollable body cannot host them.

const MIN_WIDTH = 480
/** Max width = viewport minus this margin */
const VIEWPORT_MARGIN = 48

export function useDialogEdgeResize() {
  let drag: {
    side: 'left' | 'right'
    dlg: HTMLElement
    startWidth: number
    startX: number
  } | null = null

  function onPointerDown(e: PointerEvent): void {
    const handle = e.currentTarget as HTMLElement
    const dlg = handle.closest<HTMLElement>('.am-form-dialog')
    if (!dlg) return
    drag = {
      side: handle.classList.contains('am-edge--left') ? 'left' : 'right',
      dlg,
      startWidth: dlg.getBoundingClientRect().width,
      startX: e.clientX,
    }
    handle.setPointerCapture(e.pointerId)
    // Keep text under the strip from starting a selection
    e.preventDefault()
  }

  function onPointerMove(e: PointerEvent): void {
    if (!drag) return
    const dx = e.clientX - drag.startX
    // Symmetric: both edges move by dx, the width by 2*dx
    const raw = drag.side === 'right' ? drag.startWidth + 2 * dx : drag.startWidth - 2 * dx
    const width = Math.round(Math.min(window.innerWidth - VIEWPORT_MARGIN, Math.max(MIN_WIDTH, raw)))
    drag.dlg.style.width = `${width}px`
  }

  function onPointerUp(e: PointerEvent): void {
    if (!drag) return
    ;(e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId)
    drag = null
  }

  return { onPointerDown, onPointerMove, onPointerUp }
}
