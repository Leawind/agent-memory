// DoctorCard: the at-a-glance stats strip plus the named checklist (every check gets a
// verdict row; failing ones list their raw issue lines). Mounts the card directly.
import { describe, expect, it, vi, beforeEach } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import ElementPlus from 'element-plus'
import { setMemoryUILocale } from '../../index'
import DoctorCard from './DoctorCard.vue'
import type { StatsInfo } from '../../types'

function jsonResponse(body: unknown) {
  return { ok: true, status: 200, text: () => Promise.resolve(JSON.stringify(body)) }
}

const STATS: StatsInfo = {
  path: '/tmp/m.db',
  memories: 12,
  tags: 4,
  file_size: 2048,
  schema_version: 1,
}

function mountCard(stats: StatsInfo | null = STATS) {
  return mount(DoctorCard, { props: { stats }, global: { plugins: [ElementPlus] } })
}

beforeEach(() => {
  setMemoryUILocale('zh')
})

describe('DoctorCard', () => {
  it('shows the data strip without extra requests before the first run', () => {
    const fetchMock = vi.fn()
    vi.stubGlobal('fetch', fetchMock)
    const wrapper = mountCard()
    const strip = wrapper.find('.stat-strip').text()
    expect(strip).toContain('记忆条数')
    expect(strip).toContain('12')
    expect(strip).toContain('2.0 KB')
    expect(strip).toContain('v1')
    // Running the doctor is explicit: no fetch until the button is clicked
    expect(fetchMock).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('renders a verdict row per check and lists issue lines of failing ones', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) => {
        if (String(url).includes('/api/doctor')) {
          return Promise.resolve(
            jsonResponse({
              ok: false,
              checks: [
                { id: 'integrity', ok: true, issues: [] },
                { id: 'tag_refs', ok: false, issues: ['memory_tags rows reference missing tags: 999'] },
                { id: 'future_unknown_id', ok: true, issues: [] },
              ],
            }),
          )
        }
        return Promise.resolve(jsonResponse({}))
      }),
    )
    const wrapper = mountCard()
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '运行体检')!
      .trigger('click')
    await flushPromises()
    // Summary alert counts issues and failing checks
    expect(wrapper.text()).toContain('发现 1 个问题（1/3 项检查未通过）')
    const rows = wrapper.findAll('.checks > li')
    expect(rows.length).toBe(3)
    // Known ids are localized, unknown ids degrade to the raw id
    expect(rows[0].text()).toContain('数据库完整性')
    expect(rows[1].text()).toContain('标签引用')
    expect(rows[2].text()).toContain('future_unknown_id')
    // The failing row lists its raw issues; passing rows carry none
    expect(rows[1].findAll('.issues li')[0].text()).toContain('999')
    expect(rows[0].find('.issues').exists()).toBe(false)
    expect(rows[1].find('.check-count').text()).toContain('1 个问题')
    wrapper.unmount()
  })

  it('celebrates a fully clean run with the success alert', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn((url: string | URL) =>
        String(url).includes('/api/doctor')
          ? Promise.resolve(jsonResponse({ ok: true, checks: [{ id: 'integrity', ok: true, issues: [] }] }))
          : Promise.resolve(jsonResponse({})),
      ),
    )
    const wrapper = mountCard()
    await wrapper
      .findAll('button')
      .find((b) => b.text() === '运行体检')!
      .trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('体检通过：1 项检查全部通过')
    expect(wrapper.find('.el-alert--success').exists()).toBe(true)
    wrapper.unmount()
  })
})
