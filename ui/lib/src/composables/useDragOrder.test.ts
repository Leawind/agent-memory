// useDragOrder tests. SortableJS is mocked: the DOM half of a drag (geometry, drag image, the
// animated shift) is the library's and is exercised in a real browser, while the half that is ours
// is what these tests pin down — the container and options we hand over, and that a drop becomes
// exactly one splice-move in the rows we were given.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { defineComponent, h, ref } from 'vue'
import type { SortableEvent, SortableOptions } from 'sortablejs'
import { moveRow, useDragOrder } from './useDragOrder'

interface Instance {
  el: Element
  options: SortableOptions
  destroy: ReturnType<typeof vi.fn>
}

const sortable = vi.hoisted(() => ({ created: [] as Instance[] }))

vi.mock('sortablejs', () => ({
  default: {
    create: (el: Element, options: SortableOptions) => {
      const instance = { el, options, destroy: vi.fn() }
      sortable.created.push(instance)
      return instance
    },
  },
}))

/** Mount a component that renders `rows` through the composable and reports the sortable it made */
function setup(rows: string[], options?: { item?: string; handle?: string }) {
  const list = ref(rows)
  const listEl = ref<HTMLElement | null>(null)
  const wrapper = mount(
    defineComponent({
      setup() {
        useDragOrder(listEl, list, options)
        return () =>
          h(
            'div',
            { ref: listEl },
            list.value.map((r) => h('div', { class: 'entry' }, r)),
          )
      },
    }),
  )
  const made = () => sortable.created[sortable.created.length - 1]
  /** The callback SortableJS fires with the two positions it just swapped in the DOM */
  const reportDrop = (oldIndex: number, newIndex: number) =>
    made().options.onEnd!({ oldIndex, newIndex } as SortableEvent)
  return { wrapper, list, made, reportDrop }
}

beforeEach(() => {
  sortable.created.length = 0
})

function move(rows: string[], from: number, to: number): string[] {
  const copy = [...rows]
  moveRow(copy, from, to)
  return copy
}

describe('moveRow', () => {
  it('moves a row to the target position and shifts the rows in between', () => {
    expect(move(['a', 'b', 'c', 'd'], 0, 2)).toEqual(['b', 'c', 'a', 'd'])
    expect(move(['a', 'b', 'c', 'd'], 3, 1)).toEqual(['a', 'd', 'b', 'c'])
  })

  it('leaves the list alone for a no-op or an out-of-range drag', () => {
    expect(move(['a', 'b'], 1, 1)).toEqual(['a', 'b'])
    expect(move(['a', 'b'], -1, 0)).toEqual(['a', 'b'])
    expect(move(['a', 'b'], 0, 2)).toEqual(['a', 'b'])
    expect(move(['a', 'b'], 2, 0)).toEqual(['a', 'b'])
  })
})

describe('useDragOrder', () => {
  it('hands the list to SortableJS: the grip starts drags, the gap previews the landing', () => {
    const { wrapper, made } = setup(['a', 'b'])
    expect(made().el).toBe(wrapper.element)
    expect(made().options.draggable).toBe('.entry')
    expect(made().options.handle).toBe('.entry-handle')
    expect(made().options.animation).toBe(150)
    expect(made().options.ghostClass).toBe('is-ghost')
    expect(made().options.fallbackClass).toBe('is-drag')
  })

  it('takes custom selectors for a list that is not built out of entries', () => {
    const { made } = setup(['a'], { item: '.row', handle: '.grip' })
    expect(made().options.draggable).toBe('.row')
    expect(made().options.handle).toBe('.grip')
  })

  it('follows the drop: the rows come out in the order the user left them', async () => {
    const { list, reportDrop } = setup(['a', 'b', 'c'])
    reportDrop(0, 2)
    expect(list.value).toEqual(['b', 'c', 'a'])
  })

  it('stops listening when the component is gone', () => {
    const { wrapper, made } = setup(['a', 'b'])
    wrapper.unmount()
    expect(made().destroy).toHaveBeenCalled()
  })
})
