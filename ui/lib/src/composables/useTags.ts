// Tag and scope state: the two lists plus CRUD. Failed actions throw an Error; the caller layer
// shows the toast uniformly.
import { onMounted, ref } from 'vue'
import { useApiClient } from '../api/client'
import {
  createScope as apiCreateScope,
  createTag,
  deleteTag,
  listTags,
  updateScope as apiUpdateScope,
  updateTag,
} from '../api/tags'
import type { TagDeleteMode } from '../api/tags'
import type { ScopeView, TagView } from '../types'

export function useTags() {
  const client = useApiClient()
  const rows = ref<TagView[]>([])
  const scopes = ref<ScopeView[]>([])
  const loading = ref(false)

  async function reload() {
    loading.value = true
    try {
      const data = await listTags(client)
      rows.value = data.tags ?? []
      scopes.value = data.scopes ?? []
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

  async function createScope(name: string, description: string, required?: boolean) {
    await apiCreateScope(client, name, description, required)
    await reload()
  }

  async function updateScope(name: string, description?: string, required?: boolean) {
    await apiUpdateScope(client, name, description, required)
    await reload()
  }

  onMounted(() => {
    void reload().catch(() => {}) // first-load errors are toasted by the caller (wrapped at the caller layer)
  })

  return { rows, scopes, loading, reload, create, rename, remove, createScope, updateScope }
}

export type TagsStore = ReturnType<typeof useTags>
