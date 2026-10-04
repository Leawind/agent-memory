// FormDialog shell: footer buttons, their states, and the two events. The el-form wrapper
// and slot passthrough are exercised implicitly; callers own the fields and submit logic.
import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import FormDialog from './FormDialog.vue'
import { t } from '../i18n'

// The el-dialog content is lazy-rendered: mount closed, then open via a prop change
async function mountOpened(props: { title: string } & Record<string, unknown>, slots?: Record<string, string>) {
  const wrapper = mount(FormDialog, { props: { visible: false, ...props }, slots })
  await wrapper.setProps({ visible: true })
  await wrapper.vm.$nextTick()
  return wrapper
}

describe('FormDialog', () => {
  it('renders the title, default save/cancel footer, and emits the two events', async () => {
    const wrapper = await mountOpened({ title: '测试弹窗' }, { default: '<div class="field">字段</div>' })
    const dialog = wrapper.find('.am-form-dialog')
    expect(dialog.exists()).toBe(true)
    expect(dialog.text()).toContain('测试弹窗')
    expect(dialog.text()).toContain('字段')

    const footerButtons = wrapper.findAll('.el-dialog__footer .el-button')
    expect(footerButtons).toHaveLength(2)
    expect(footerButtons[0].text()).toBe(t('common.cancel'))
    expect(footerButtons[1].text()).toBe(t('common.save'))
    // Submit defaults to primary; cancel is plain
    expect(footerButtons[1].classes()).toContain('el-button--primary')

    await footerButtons[0].trigger('click')
    expect(wrapper.emitted('update:visible')).toEqual([[false]])
    await footerButtons[1].trigger('click')
    expect(wrapper.emitted('submit')).toHaveLength(1)
    expect(wrapper.emitted('update:visible')).toHaveLength(1)
    wrapper.unmount()
  })

  it('supports a custom danger submit button and the submitDisabled guard', async () => {
    const wrapper = await mountOpened({
      title: '删除',
      submitText: t('common.delete'),
      submitType: 'danger',
      submitDisabled: true,
    })
    const submit = wrapper.findAll('.el-dialog__footer .el-button')[1]
    expect(submit.text()).toBe(t('common.delete'))
    expect(submit.classes()).toContain('el-button--danger')
    expect(submit.attributes('disabled')).toBeDefined()
    wrapper.unmount()
  })

  it('shows the loading state while saving', async () => {
    const wrapper = await mountOpened({ title: '编辑', saving: true })
    const submit = wrapper.findAll('.el-dialog__footer .el-button')[1]
    expect(submit.classes()).toContain('is-loading')
    wrapper.unmount()
  })
})
