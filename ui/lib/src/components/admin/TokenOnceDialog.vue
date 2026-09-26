<template>
  <!-- 新建 / 重置成功：token 明文仅此一次展示 -->
  <el-dialog v-model="visible" :title="title" :width="compact ? '96%' : '560px'">
    <p>{{ t('access.created') }}</p>
    <code class="new-token">{{ token }}</code>
    <template #footer>
      <!-- 宿主注入 onIdentityToken 时提供一键保存（独立站点壳存入多身份令牌表并切换） -->
      <el-button v-if="config.onIdentityToken" type="primary" @click="saveToBrowser">
        {{ t('access.saveToBrowser') }}
      </el-button>
      <el-button
        type="primary"
        @click="
          () => {
            copyToken(token)
            visible = false
          }
        "
      >
        {{ t('access.copyToken') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { t } from '../../i18n'
import { useMemoryConfig } from '../../config'
import { toastError, toastSuccess } from '../../toast'

defineProps<{
  /** compact（<960px）时对话框加宽到 96% */
  compact: boolean
}>()

const config = useMemoryConfig()

const visible = ref(false)
const title = ref('')
const identityName = ref('')
const token = ref('')

/** 展示一次性 token（供父层在创建/重置成功后调用）。 */
function show(newTitle: string, name: string, newToken: string): void {
  title.value = newTitle
  identityName.value = name
  token.value = newToken
  visible.value = true
}

defineExpose({ show })

// 「保存到本浏览器」：把一次性 token 交给宿主的令牌表（独立站点壳会记住并切换）。
// 按钮仅在宿主注入 onIdentityToken 时渲染，这里非空调用。
async function saveToBrowser(): Promise<void> {
  if (!identityName.value) return
  try {
    await config.onIdentityToken!(identityName.value, token.value)
    visible.value = false
    toastSuccess(t('access.savedToBrowser', { name: identityName.value }))
  } catch (e) {
    toastError(e instanceof Error ? e.message : String(e))
  }
}

async function copyToken(value: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(value)
    toastSuccess(t('access.copied'))
  } catch {
    toastError(value)
  }
}
</script>

<style scoped>
.new-token {
  display: block;
  margin-top: 8px;
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--el-fill-color);
  font-size: 13px;
  word-break: break-all;
  user-select: all;
}
</style>
