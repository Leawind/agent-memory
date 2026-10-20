<template>
  <el-dialog
    :model-value="visible"
    :title="t('predicates.title')"
    width="min(820px, 96vw)"
    append-to-body
    @update:model-value="emit('update:visible', $event)"
  >
    <p>{{ t('predicates.hint') }}</p>
    <el-table :data="rows" size="small" @row-click="edit">
      <el-table-column prop="name" :label="t('predicates.name')" />
      <el-table-column prop="predicate" :label="t('predicates.expression')" />
      <el-table-column prop="description" :label="t('tags.descLabel')" />
      <el-table-column width="180"
        ><template #default="{ row }"
          ><el-button size="small" @click.stop="apply(row)">{{ t('predicates.search') }}</el-button
          ><el-button v-if="canManage" size="small" @click.stop="remove(row)">{{
            t('common.delete')
          }}</el-button></template
        ></el-table-column
      >
    </el-table>
    <el-button class="new-predicate" @click="reset">{{ t('predicates.create') }}</el-button>
    <el-form label-position="top" :disabled="busy || !canManage" @submit.prevent>
      <el-form-item :label="t('predicates.name')"
        ><el-input v-model="draft.name" :disabled="editing" :maxlength="100"
      /></el-form-item>
      <el-form-item :label="t('predicates.expression')"
        ><el-input v-model="draft.predicate" :maxlength="32768" placeholder="a&amp;!b | @other"
      /></el-form-item>
      <el-form-item :label="t('tags.descLabel')"
        ><el-input v-model="draft.description" :maxlength="512"
      /></el-form-item>
    </el-form>
    <p>{{ t('predicates.namesHint') }}</p>
    <template #footer
      ><el-button @click="emit('update:visible', false)">{{ t('access.close') }}</el-button
      ><el-button type="primary" :loading="busy" :disabled="!valid || !canManage" @click="save">{{
        t('common.save')
      }}</el-button></template
    >
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useApiClient } from '../api/client'
import { t } from '../i18n'
import { run } from './admin/caps'
import type { NamedPredicate, WhoAmI } from '../types'
const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ 'update:visible': [boolean]; changed: []; apply: [expression: string] }>()
const api = useApiClient()
const rows = ref<NamedPredicate[]>([])
const canManage = ref(false)
const busy = ref(false)
const editing = ref(false)
const empty = (): NamedPredicate => ({ name: '', predicate: '', description: '' })
const draft = ref(empty())
const valid = computed(
  () => /^[\p{L}\p{N}_][\p{L}\p{N}_.-]{0,99}$/u.test(draft.value.name.trim()) && !!draft.value.predicate.trim(),
)
let generation = 0
function reset(): void {
  draft.value = empty()
  editing.value = false
}
function edit(row: NamedPredicate): void {
  if (busy.value) return
  draft.value = { ...row }
  editing.value = true
}
function apply(row: NamedPredicate): void {
  emit('apply', `@${row.name}`)
  emit('update:visible', false)
}
async function load(): Promise<void> {
  const current = generation
  await run(async () => {
    const [listing, who] = await Promise.all([
      api.get<{ predicates: NamedPredicate[] }>('/api/predicates'),
      api.get<WhoAmI>('/api/whoami'),
    ])
    if (!props.visible || current !== generation) return
    rows.value = listing.predicates
    canManage.value = who.permissions.predicate_manage === true
  })
}
watch(
  () => props.visible,
  (visible) => {
    generation++
    if (visible) {
      reset()
      rows.value = []
      canManage.value = false
      void load()
    }
  },
  { immediate: true },
)
async function save(): Promise<void> {
  if (busy.value || !valid.value || !canManage.value) return
  busy.value = true
  try {
    await run(async () => {
      await api.put('/api/predicates', draft.value)
      emit('changed')
      await load()
      editing.value = true
    })
  } finally {
    busy.value = false
  }
}
async function remove(row: NamedPredicate): Promise<void> {
  if (busy.value || !canManage.value) return
  busy.value = true
  try {
    await run(async () => {
      await api.del('/api/predicates', { name: row.name })
      reset()
      emit('changed')
      await load()
    })
  } finally {
    busy.value = false
  }
}
</script>
<style scoped>
.new-predicate {
  margin: 16px 0;
}
</style>
