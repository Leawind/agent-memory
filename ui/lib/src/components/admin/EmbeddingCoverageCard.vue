<template>
  <!-- 语义搜索覆盖率与补跑：backfill 是 admin 端点（有界批量，按钮内循环直到清零） -->
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.embeddingCard') }}</span>
        <el-button size="small" :icon="Refresh" :loading="backfilling" @click="runBackfill">
          {{ t('access.runBackfill') }}
        </el-button>
      </div>
    </template>
    <template v-if="stats?.embedding?.enabled">
      <el-descriptions :column="compact ? 1 : 2" border>
        <el-descriptions-item :label="t('access.embeddingModel')">
          {{ stats.embedding.model ?? '—' }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('access.embeddingCoverage')">
          {{ stats.embedding.embedded ?? 0 }} / {{ coverageTotal }}
        </el-descriptions-item>
      </el-descriptions>
      <el-alert
        v-if="(stats.embedding.pending ?? 0) > 0"
        :title="t('access.embeddingPending', { count: stats.embedding.pending })"
        type="warning"
        show-icon
        :closable="false"
      />
    </template>
    <el-alert v-else :title="t('access.embeddingDisabled')" type="info" show-icon :closable="false" />
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Refresh } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import { useAdmin } from '../../composables/useAdmin'
import type { StatsInfo } from '../../types'
import { run } from './caps'

const props = defineProps<{
  stats: StatsInfo | null
  /** compact（<960px）时描述列表单列 */
  compact: boolean
}>()

// 语义搜索覆盖率：统计随父层 load 一并拉取（/api/stats 只需 read，但补跑按钮仅 admin 可用）
const api = useApiClient()
const { backfilling, backfill } = useAdmin()

const emit = defineEmits<{ changed: [] }>()

// 覆盖率分母 = 已向量化 + 待补跑（未启用时为 0）
const coverageTotal = computed(() => (props.stats?.embedding?.embedded ?? 0) + (props.stats?.embedding?.pending ?? 0))

/** 补跑完成后给结果反馈（成功条数为 0 也算成功——本就无待办），并触发父层刷新覆盖率 */
async function runBackfill(): Promise<void> {
  const total = await run(backfill)
  if (total === undefined) return
  emit('changed')
  if (total === 0) toastSuccess(t('access.embeddingUpToDate'))
  else toastSuccess(t('access.embeddingDone', { count: total }))
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
</style>
