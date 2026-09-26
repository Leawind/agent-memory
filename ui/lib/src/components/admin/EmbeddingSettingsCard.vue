<template>
  <!-- 语义搜索配置：OpenAI 兼容 /embeddings（云端或本地 Ollama）；服务不可用时自动回退关键词 -->
  <el-card shadow="never">
    <template #header>{{ t('access.embeddingTitle') }}</template>
    <el-alert :title="t('access.embeddingHint')" type="info" show-icon :closable="false" class="settings-hint" />
    <el-form label-position="top" @submit.prevent>
      <el-form-item :label="t('access.embeddingEnabledLabel')">
        <el-switch v-model="enabled" />
      </el-form-item>
      <el-form-item :label="t('access.embeddingBaseUrl')">
        <el-input v-model="baseUrl" placeholder="https://api.siliconflow.cn/v1 或 http://127.0.0.1:11434/v1" />
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
  </el-card>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { t } from '../../i18n'
import { useApiClient } from '../../api/client'
import { toastSuccess } from '../../toast'
import { run } from './caps'

const props = defineProps<{
  embeddingEnabled: boolean
  embeddingBaseUrl: string
  embeddingModel: string
  embeddingApiKey: string
}>()

// 本地编辑态，随父层 load 刷新
const enabled = ref(props.embeddingEnabled)
const baseUrl = ref(props.embeddingBaseUrl)
const model = ref(props.embeddingModel)
const apiKey = ref(props.embeddingApiKey)
watch(
  () => [props.embeddingEnabled, props.embeddingBaseUrl, props.embeddingModel, props.embeddingApiKey],
  ([e, u, m, k]) => {
    enabled.value = e
    baseUrl.value = u
    model.value = m
    apiKey.value = k
  },
)

const api = useApiClient()
const saving = ref(false)
const test = ref<null | { ok: boolean; dim?: number; elapsed_ms?: number; error?: string }>(null)

// 保存配置后立刻用服务端配置做连通性测试
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
    if (!enabled.value) return // 关闭状态无需测试
    test.value = await run(() => api.post('/api/embeddings/test', {}))
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.save-row {
  margin-top: 12px;
  display: flex;
  justify-content: flex-end;
}
</style>
