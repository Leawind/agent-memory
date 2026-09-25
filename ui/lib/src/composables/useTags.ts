// 标签面板的状态：列表与 CRUD。动作失败时抛出 Error，由面板层统一 toast。
import { onMounted, ref } from 'vue'
import { useApiClient } from '../api/client'
import { createTag, deleteTag, listTags, updateTag } from '../api/tags'
import type { TagDeleteMode } from '../api/tags'
import type { TagView } from '../types'

export function useTags() {
  const client = useApiClient()
  const rows = ref<TagView[]>([])
  const loading = ref(false)

  async function reload() {
    loading.value = true
    try {
      const data = await listTags(client)
      rows.value = data.tags ?? []
    } finally {
      loading.value = false
    }
  }

  async function create(name: string, description: string) {
    await createTag(client, name, description)
    await reload()
  }

  async function rename(oldName: string, newName: string, description: string) {
    await updateTag(client, oldName, newName, description)
    await reload()
  }

  async function remove(name: string, mode: TagDeleteMode) {
    await deleteTag(client, name, mode)
    await reload()
  }

  onMounted(() => {
    void reload().catch(() => {}) // 首屏错误由调用方 toast（面板层包装）
  })

  return { rows, loading, reload, create, rename, remove }
}

export type TagsStore = ReturnType<typeof useTags>
