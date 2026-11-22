<template>
  <!-- Backup export/import: admin-only endpoints (/api/export is intercepted by the http layer as an attachment download) -->
  <el-card shadow="never">
    <template #header>{{ t('access.backupCard') }}</template>
    <div class="rows">
      <div class="row">
        <TagExprInput
          v-model="exportPredicate"
          class="expr"
          :tags="tagNames"
          :placeholder="t('access.predicatePlaceholder')"
          :help="t('memories.tagExprHelp')"
        />
        <el-button :icon="Download" :loading="exporting" @click="run(() => exportData(exportPredicate || undefined))">
          {{ t('access.export') }}
        </el-button>
      </div>
      <div class="row">
        <TagExprInput
          v-model="importPredicate"
          class="expr"
          :tags="tagNames"
          :placeholder="t('access.predicatePlaceholder')"
          :help="t('memories.tagExprHelp')"
        />
        <el-checkbox v-model="merge">{{ t('access.merge') }}</el-checkbox>
        <el-button :icon="UploadFilled" :loading="importing" @click="importInput?.click()">
          {{ t('access.import') }}
        </el-button>
        <el-tooltip :content="t('access.importHint')" placement="top">
          <el-icon class="am-info"><InfoFilled /></el-icon>
        </el-tooltip>
        <input
          ref="importInput"
          type="file"
          accept="application/json,.json"
          style="display: none"
          @change="onImportFile"
        />
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { Download, InfoFilled, UploadFilled } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { toastError, toastSuccess } from '../../toast'
import { useAdmin } from '../../composables/useAdmin'
import { listTags } from '../../api/tags'
import { useApiClient } from '../../api/client'
import { run } from './caps'
import TagExprInput from '../TagExprInput.vue'

const { exporting, importing, exportData, importFile } = useAdmin()
const client = useApiClient()

const exportPredicate = ref('')
const importPredicate = ref('')
const merge = ref(false)
/** Known tag names for the expression completion; a failed load just disables suggestions */
const tagNames = ref<string[]>([])

onMounted(async () => {
  try {
    const resp = await listTags(client)
    tagNames.value = resp.tags.map((tag) => tag.name)
  } catch {
    tagNames.value = []
  }
})

const importInput = ref<HTMLInputElement | null>(null)

function onImportFile(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  importFile(file, { merge: merge.value, predicate: importPredicate.value || undefined })
    .then((imported) => {
      const unclassified = imported.unclassified_memories ?? 0
      const key =
        unclassified > 0
          ? 'access.importedUnclassified'
          : imported.skipped_memories > 0
            ? 'access.importedSkipped'
            : 'access.imported'
      toastSuccess(
        t(key, {
          memories: imported.imported_memories,
          tags: imported.imported_tags,
          skipped: imported.skipped_memories,
          unclassified,
        }),
      )
    })
    .catch((err: unknown) => {
      toastError(err instanceof Error ? err.message : String(err))
    })
}
</script>

<style scoped>
.rows {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.row {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  align-items: center;
}
.expr {
  flex: 1;
  min-width: 220px;
  max-width: 480px;
}
</style>
