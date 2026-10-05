// TagExprInput completion behavior: suggestions for the token being typed, token-scoped
// replacement (longer expressions keep their surroundings), quoted-name re-closing, and the
// apply/close keyboard flow. The harness feeds each emitted value back through props — the
// panel's v-model contract. happy-dom input selection defaults to end-of-value, which is
// exactly the while-typing scenario.
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import TagExprInput from './TagExprInput.vue'

const TAGS = ['rust', 'web', '项目', 'rust & life']

function mounted(value = '') {
  return mount(TagExprInput, {
    // focus()/focusin only fire on elements attached to the document
    attachTo: document.body,
    props: { modelValue: value, tags: TAGS, placeholder: 'expr', help: 'help text' },
  })
}

function lastUpdate(wrapper: ReturnType<typeof mounted>): string {
  const events = wrapper.emitted('update:modelValue') ?? []
  return String(events.at(-1)?.[0])
}

async function type(wrapper: ReturnType<typeof mounted>, value: string) {
  const input = wrapper.find('input')
  input.element.value = value
  await input.setValue(value)
  // Emulate the panel's v-model: the emitted value flows back as the prop
  await wrapper.setProps({ modelValue: lastUpdate(wrapper) })
}

describe('TagExprInput', () => {
  it('suggests tags for the token being typed, ranked by prefix', async () => {
    const wrapper = mounted()
    await type(wrapper, 'ru')
    const names = wrapper.findAll('.am-expr-item').map((i) => i.text())
    expect(names).toEqual(['rust', 'rust & life'])
    expect(names).not.toContain('web')
    wrapper.unmount()
  })

  it('enter completes the highlighted suggestion and applies', async () => {
    const wrapper = mounted()
    await type(wrapper, 'ru')
    await wrapper.find('input').trigger('keydown.enter')
    expect(lastUpdate(wrapper)).toBe('rust')
    expect(wrapper.emitted('apply')).toHaveLength(1)
    expect(wrapper.find('.am-expr-pop').exists()).toBe(false)
    wrapper.unmount()
  })

  it('completion replaces only the trailing token, keeping the rest of the expression', async () => {
    const wrapper = mounted()
    await type(wrapper, '(ru&we')
    await wrapper.find('input').trigger('keydown.enter')
    expect(lastUpdate(wrapper)).toBe('(ru&web')
    wrapper.unmount()
  })

  it('completing a name inside a quote re-closes the quote', async () => {
    const wrapper = mounted()
    await type(wrapper, "'ru")
    // Two suggestions share the prefix: move to 'rust & life' before completing
    await wrapper.find('input').trigger('keydown.down')
    await wrapper.find('input').trigger('keydown.enter')
    expect(lastUpdate(wrapper)).toBe("'rust & life'")
    wrapper.unmount()
  })

  it('escape closes the dropdown; a later enter applies the raw expression', async () => {
    const wrapper = mounted()
    await type(wrapper, 'ru')
    await wrapper.find('input').trigger('keydown.esc')
    expect(wrapper.find('.am-expr-pop').exists()).toBe(false)
    await wrapper.find('input').trigger('keydown.enter')
    expect(wrapper.emitted('apply')).toHaveLength(1)
    expect(lastUpdate(wrapper)).toBe('ru')
    wrapper.unmount()
  })

  it('focusing an empty box offers the whole taxonomy', async () => {
    const wrapper = mounted()
    ;(wrapper.find('input').element as HTMLInputElement).focus()
    await wrapper.vm.$nextTick()
    expect(wrapper.findAll('.am-expr-item').map((i) => i.text())).toEqual(TAGS)
    wrapper.unmount()
  })

  it('arrow keys move the active suggestion', async () => {
    const wrapper = mounted()
    await type(wrapper, 'ru')
    const items = wrapper.findAll('.am-expr-item')
    expect(items.length).toBe(2)
    expect(items[0].classes()).toContain('active')
    await wrapper.find('input').trigger('keydown.down')
    expect(items[1].classes()).toContain('active')
    await wrapper.find('input').trigger('keydown.up')
    expect(items[0].classes()).toContain('active')
    wrapper.unmount()
  })
})
