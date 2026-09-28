<template>
  <el-config-provider :locale="elementPlusLocale">
    <el-dialog
      :model-value="visible"
      :title="t('ops.title')"
      width="min(480px, calc(100vw - 24px))"
      @update:model-value="emit('update:visible', $event)"
    >
      <div class="stats-row">
        <el-statistic :title="t('ops.statMemories')" :value="stats.memories ?? 0" />
        <el-statistic :title="t('ops.statTags')" :value="stats.tags ?? 0" />
        <el-statistic :title="t('ops.statSize')" :value="sizeText" />
      </div>
      <el-descriptions :column="1" border>
        <el-descriptions-item :label="t('ops.version')">{{ stats.version ?? version }}</el-descriptions-item>
        <el-descriptions-item :label="t('ops.schemaVersion')">{{ stats.schema_version ?? '—' }}</el-descriptions-item>
        <el-descriptions-item :label="t('ops.path')">{{ stats.path ?? '—' }}</el-descriptions-item>
      </el-descriptions>
    </el-dialog>
  </el-config-provider>
</template>

<script setup lang="ts">
import { watch } from 'vue'
import { t } from '../i18n'
import { elementPlusLocale } from '../i18n/elementPlus'
import { useOps } from '../composables/useOps'
import { toastError } from '../toast'

const props = defineProps<{
  /** Visibility (controlled by host entry points such as a title click) */ visible: boolean
}>()
const emit = defineEmits<{ 'update:visible': [value: boolean] }>()

const { stats, version, sizeText, reload } = useOps()

// Refetch on every open: the overview is always fresh when clicked, no manual refresh needed
watch(
  () => props.visible,
  (open) => {
    if (open) reload().catch((e: unknown) => toastError(e instanceof Error ? e.message : String(e)))
  },
)
</script>

<style scoped>
.stats-row {
  display: flex;
  gap: 36px;
  margin-bottom: 16px;
}
</style>
