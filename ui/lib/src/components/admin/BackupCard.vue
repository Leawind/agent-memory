<template>
  <!-- 备份导入导出：admin 专属端点（/api/export 由 http 层拦截为附件下载） -->
  <el-card shadow="never">
    <template #header>{{ t('access.backupCard') }}</template>
    <div class="actions">
      <el-button :icon="Download" :loading="exporting" @click="run(exportData)">
        {{ t('access.export') }}
      </el-button>
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
  </el-card>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { Download, InfoFilled, UploadFilled } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { toastError, toastSuccess } from '../../toast'
import { useAdmin } from '../../composables/useAdmin'
import { run } from './caps'

const { exporting, importing, exportData, importFile } = useAdmin()

const importInput = ref<HTMLInputElement | null>(null)

function onImportFile(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  importFile(file)
    .then((imported) => {
      toastSuccess(t('access.imported', { memories: imported.imported_memories, tags: imported.imported_tags }))
    })
    .catch((err: unknown) => {
      toastError(err instanceof Error ? err.message : String(err))
    })
}
</script>

<style scoped>
.actions {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  align-items: center;
}
</style>
