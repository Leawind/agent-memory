<template>
  <el-drawer
    :model-value="visible"
    :title="t('drawer.title', { id: detail?.id ?? memoryId ?? '' })"
    :size="size"
    @update:model-value="emit('update:visible', $event)"
  >
    <template v-if="detail">
      <h3 class="detail-summary">{{ detail.summary }}</h3>
      <div class="detail-tags">
        <el-tag v-for="t in detail.tags" :key="t" size="small" class="am-tag">{{ t }}</el-tag>
      </div>
      <el-divider />
      <div class="detail-toolbar">
        <el-radio-group v-model="detailTab" size="small">
          <el-radio-button value="rendered">{{ t('drawer.rendered') }}</el-radio-button>
          <el-radio-button value="source">{{ t('drawer.source') }}</el-radio-button>
        </el-radio-group>
      </div>
      <MarkdownView v-if="detailTab === 'rendered'" :source="detail.content" />
      <pre v-else class="detail-content">{{ detail.content }}</pre>
    </template>
  </el-drawer>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { toastError, toastSuccess } from '../toast'
import { useApiClient } from '../api/client'
import { getMemory } from '../api/memories'
import { t } from '../i18n'
import type { MemoryFull } from '../types'
import MarkdownView from './MarkdownView.vue'

const props = defineProps<{
  visible: boolean
  memoryId: string | null
  /** 抽屉宽度；窄容器由面板传入 '100%' */
  size?: string
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

const client = useApiClient()
const detail = ref<MemoryFull | null>(null)
// 详情抽屉视图：默认渲染 Markdown，可切回源码
const detailTab = ref<'rendered' | 'source'>('rendered')

watch(
  () => [props.visible, props.memoryId] as const,
  async ([open]) => {
    if (!open || !props.memoryId) return
    detailTab.value = 'rendered'
    try {
      detail.value = await getMemory(client, props.memoryId)
    } catch (e: unknown) {
      toastError(e instanceof Error ? e.message : String(e))
      emit('update:visible', false)
    }
  },
)
</script>

<style scoped>
.detail-toolbar {
  margin-bottom: 12px;
}
.detail-summary {
  margin: 0 0 10px;
}
.detail-tags {
  margin-bottom: 6px;
}
.detail-content {
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
  line-height: 1.7;
  margin: 0;
}
</style>
