// useDialogEdgeResize behavior test through TagDialog: the dialog stays centered while
// resizing — both edges move symmetrically, so the width changes by twice the pointer
// delta. Layout is stubbed — happy-dom computes no boxes — so the drag math is verified
// deterministically against fixed rectangles.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { DOMWrapper, mount, flushPromises } from '@vue/test-utils'
import TagDialog from '../components/TagDialog.vue'
import type { TagView } from '../types'

const tag: TagView = { name: 'rust', count: 3, description: '语言' }

function jsonResponse(body: unknown = null) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

function rect(left: number, width: number): DOMRect {
  return { left, right: left + width, width, top: 0, height: 900 } as DOMRect
}

/** Wire deterministic geometry: dialog at x=100..740 (640 wide) inside a 1280-wide container
 * centered at x=640 (where the flex centering puts it after any width change) */
function stubGeometry(dialog: Element, container: Element): void {
  dialog.getBoundingClientRect = () => rect(100, 640)
  container.getBoundingClientRect = () => rect(0, 1280)
}

async function openDialog() {
  const wrapper = mount(TagDialog, { props: { visible: false, tag }, attachTo: document.body })
  await wrapper.setProps({ visible: true })
  await flushPromises()
  return wrapper
}

describe('useDialogEdgeResize', () => {
  beforeEach(() => {
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve(jsonResponse())),
    )
    document.body.innerHTML = ''
    // happy-dom's default viewport varies; pin it so the max-width clamp is deterministic
    Object.defineProperty(window, 'innerWidth', { value: 1024, configurable: true })
  })

  it('dragging the right edge grows the width symmetrically by twice the pointer delta', async () => {
    const wrapper = await openDialog()
    const dialog = document.querySelector('.am-form-dialog') as HTMLElement
    stubGeometry(dialog, dialog.parentElement as Element)
    const handle = dialog.querySelector('.am-edge--right') as HTMLElement
    handle.setPointerCapture = () => {}

    await new DOMWrapper(handle).trigger('pointerdown', { clientX: 740, pointerId: 1 })
    await new DOMWrapper(handle).trigger('pointermove', { clientX: 800, pointerId: 1 })
    // Pointer moved +60: width +120, both edges move +60 around the fixed center (640)
    expect(dialog.style.width).toBe('760px')
    expect(dialog.style.translate).toBe('')
    expect(640 - 760 / 2).toBe(260)
    expect(640 + 760 / 2).toBe(1020)
    wrapper.unmount()
  })

  it('dragging the left edge grows the width symmetrically by twice the pointer delta', async () => {
    const wrapper = await openDialog()
    const dialog = document.querySelector('.am-form-dialog') as HTMLElement
    stubGeometry(dialog, dialog.parentElement as Element)
    const handle = dialog.querySelector('.am-edge--left') as HTMLElement
    handle.setPointerCapture = () => {}

    await new DOMWrapper(handle).trigger('pointerdown', { clientX: 100, pointerId: 1 })
    await new DOMWrapper(handle).trigger('pointermove', { clientX: 40, pointerId: 1 })
    // Pointer moved -60: width +120 around the fixed center
    expect(dialog.style.width).toBe('760px')
    expect(dialog.style.translate).toBe('')
    wrapper.unmount()
  })

  it('clamps at the bounds', async () => {
    const wrapper = await openDialog()
    const dialog = document.querySelector('.am-form-dialog') as HTMLElement
    stubGeometry(dialog, dialog.parentElement as Element)
    const handle = dialog.querySelector('.am-edge--right') as HTMLElement
    handle.setPointerCapture = () => {}

    await new DOMWrapper(handle).trigger('pointerdown', { clientX: 740, pointerId: 1 })
    // Way past the viewport-minus-margin cap (pinned innerWidth 1024 - 48 = 976)
    await new DOMWrapper(handle).trigger('pointermove', { clientX: 1400, pointerId: 1 })
    expect(dialog.style.width).toBe('976px')

    // Dragging back in the same gesture unclamps and runs to the min bound
    await new DOMWrapper(handle).trigger('pointermove', { clientX: 500, pointerId: 1 })
    expect(dialog.style.width).toBe('480px')
    wrapper.unmount()
  })

  it('pointerup ends the drag: later moves change nothing', async () => {
    const wrapper = await openDialog()
    const dialog = document.querySelector('.am-form-dialog') as HTMLElement
    stubGeometry(dialog, dialog.parentElement as Element)
    const handle = dialog.querySelector('.am-edge--right') as HTMLElement
    handle.setPointerCapture = () => {}
    handle.releasePointerCapture = () => {}

    await new DOMWrapper(handle).trigger('pointerdown', { clientX: 740, pointerId: 1 })
    await new DOMWrapper(handle).trigger('pointermove', { clientX: 800, pointerId: 1 })
    await new DOMWrapper(handle).trigger('pointerup', { clientX: 800, pointerId: 1 })
    await new DOMWrapper(handle).trigger('pointermove', { clientX: 1000, pointerId: 1 })
    expect(dialog.style.width).toBe('760px')
    wrapper.unmount()
  })
})
