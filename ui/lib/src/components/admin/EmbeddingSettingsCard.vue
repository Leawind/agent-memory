<template>
  <!-- 语义搜索配置：OpenAI 兼容 /embeddings（云端或本地 Ollama）；服务不可用时自动回退关键词。
       向量覆盖率与补跑并入本卡（原底部独立卡已删）：配置与覆盖率同屏，启用后即可见 -->
  <el-card shadow="never">
    <template #header>{{ t('access.embeddingTitle') }}</template>
    <el-alert :title="t('access.embeddingHint')" type="info" show-icon :closable="false" class="settings-hint" />
    <div class="enable-row">
      <span class="enable-label">{{ t('access.embeddingEnabledLabel') }}</span>
      <el-switch v-model="enabled" />
      <span class="enable-hint">{{ t('access.embeddingEnabledHint') }}</span>
    </div>
    <el-form label-position="top" @submit.prevent>
      <el-form-item :label="t('access.embeddingBaseUrl')">
        <el-input v-model="baseUrl" :placeholder="t('access.embeddingBaseUrlPlaceholder')" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingModelLabel')">
        <el-input v-model="model" placeholder="BAAI/bge-m3 / bge-m3 / nomic-embed-text" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingApiKeyLabel')">
        <el-input v-model="apiKey" show-password :placeholder="t('access.embeddingApiKeyPlaceholder')" />
      </el-form-item>
    </el-form>
    <div class="save-row">
      <el-button type="primary" :loading="saving" @click="saveAndTest">
        {{ t('access.embeddingSaveTest') }}
      </el-button>
    </div>
    <el-alert
      v-if="test?.ok"
      :title="t('access.embeddingTestOk', { dim: test.dim ?? 0, ms: test.elapsed_ms ?? 0 })"
      type="success"
      show-icon
      :closable="false"
      class="settings-hint"
    />
    <el-alert
      v-else-if="test && !test.ok"
      :title="t('access.embeddingTestFail', { error: test.error ?? '' })"
      type="error"
      show-icon
      :closable="false"
      class="settings-hint"
    />

    <!-- 向量覆盖率与补跑（admin 端点，按钮内循环直到清零）：统计随父层 load 拉取，未启用时不渲染 -->
    <template v-if="coverage">
      <el-divider class="coverage-divider" />
      <div class="coverage-header">
        <span class="coverage-title">{{ t('access.embeddingCoverageTitle') }}</span>
        <el-button size="small" :icon="Refresh" :loading="backfilling" @click="runBackfill">
          {{ t('access.runBackfill') }}
        </el-button>
      </div>
      <el-descriptions size="small" :column="compact ? 1 : 2">
        <el-descriptions-item :label="t('access.embeddingModel')">
          <code class="coverage-model">{{ coverage.model ?? '—' }}</code>
        </el-descriptions-item>
        <el-descriptions-item :label="t('access.embeddingCoverage')">
          {{ coverage.embedded ?? 0 }} / {{ coverageTotal }}
        </el-descriptions-item>
      </el-descriptions>
      <el-alert
        v-if="(coverage.pending ?? 0) > 0"
        :title="t('access.embeddingPending', { count: coverage.pending })"
        type="warning"
        show-icon
        :closable="false"
        class="settings-hint"
      />
    </template>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Refresh } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import { useAdmin } from '../../composables/useAdmin'
import type { EmbeddingCoverage } from '../../types'
import { run } from './caps'

const props = defineProps<{
  embeddingEnabled: boolean
  embeddingBaseUrl: string
  embeddingModel: string
  embeddingApiKey: string
  /** 向量覆盖率统计（随父层 load 拉取）；未启用或未加载时不展示覆盖率小节 */
  stats: import('../../types').StatsInfo | null
  /** compact（<960px）时覆盖率描述列表单列 */
  compact: boolean
}>()

const emit = defineEmits<{ changed: [] }>()

// 本地编辑态，随父层 load 刷新
const enabled = ref(props.embeddingEnabled)
const baseUrl = ref(props.embeddingBaseUrl)
const model = ref(props.embeddingModel)
const apiKey = ref(props.embeddingApiKey)
watch(
  () => [props.embeddingEnabled, props.embeddingBaseUrl, props.embeddingModel, props.embeddingApiKey] as const,
  ([e, u, m, k]) => {
    enabled.value = e
    baseUrl.value = u
    model.value = m
    apiKey.value = k
  },
)

// 服务启用才展示覆盖率小节（stats 未加载时也不渲染，避免闪现「未启用」）
const coverage = computed<EmbeddingCoverage | undefined>(() =>
  props.stats?.embedding?.enabled ? props.stats.embedding : undefined,
)
// 覆盖率分母 = 已向量化 + 待补跑
const coverageTotal = computed(() => (coverage.value?.embedded ?? 0) + (coverage.value?.pending ?? 0))

const api = useApiClient()
const { backfilling, backfill } = useAdmin()
const saving = ref(false)
// run 失败时返回 undefined（已 toast），test 需容纳三种态：null=未测、undefined=测试失败、对象=结果
const test = ref<null | undefined | { ok: boolean; dim?: number; elapsed_ms?: number; error?: string }>(null)

// 保存配置后立刻用服务端配置做连通性测试；changed 让父层重拉统计，覆盖率小节随即更新
async function saveAndTest(): Promise<void> {
  saving.value = true
  test.value = null
  try {
    const saved = await run(() =>
      api.put('/api/settings', {
        embedding_enabled: enabled.value,
        embedding_base_url: baseUrl.value,
        embedding_model: model.value,
        embedding_api_key: apiKey.value,
      }),
    )
    if (saved === undefined) return // 保存失败已 toast
    toastSuccess(t('access.saved'))
    emit('changed')
    if (!enabled.value) return // 关闭状态无需测试
    test.value = await run(() => api.post('/api/embeddings/test', {}))
  } finally {
    saving.value = false
  }
}

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
.enable-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 18px;
}
.enable-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}
.enable-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.save-row {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
.coverage-divider {
  margin: 20px 0 16px;
}
.coverage-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.coverage-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}
.coverage-model {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
