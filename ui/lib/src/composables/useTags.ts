// Tag panel state: list and CRUD. Failed actions throw an Error; the panel layer shows the toast uniformly.
import { onMounted, ref } from 'vue'
import { useApiClient } from '../api/client'
import { createTag, deleteTag, listTags, updateTag } from '../api/tags'
import type { TagDeleteMode } from '../api/tags'
import type { TagView } from '../types'

export function useTags() {
  const client = useApiClient()
  const rows = ref<TagView[]>([])
  const loading = ref(false)
  /** Tag-name regex filter (executed server-side); empty string = no filter */
  const filter = ref('')

  async function reload() {
    loading.value = true
    try {
      const data = await listTags(client, filter.value.trim() || undefined)
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
    void reload().catch(() => {}) // first-load errors are toasted by the caller (wrapped at the panel layer)
  })

  return { rows, loading, filter, reload, create, rename, remove }
}

export type TagsStore = ReturnType<typeof useTags>
