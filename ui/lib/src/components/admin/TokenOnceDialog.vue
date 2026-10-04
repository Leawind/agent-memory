<template>
  <!-- After create / reset: the token is shown in plaintext only once -->
  <el-dialog v-model="visible" :title="title" :width="compact ? '96%' : '560px'">
    <p>{{ t('access.created') }}</p>
    <div class="token-row">
      <code class="new-token">{{ token }}</code>
      <el-button
        class="copy-btn"
        circle
        :title="t('access.copyToken')"
        :aria-label="t('access.copyToken')"
        @click="copyToken(token)"
      >
        <el-icon><Check v-if="justCopied" /><CopyDocument v-else /></el-icon>
      </el-button>
    </div>
    <template #footer>
      <el-button @click="visible = false">{{ t('access.close') }}</el-button>
      <!-- One-click save when the host injects onIdentityToken (the standalone shell stores it in its multi-identity token table and switches) -->
      <el-button v-if="config.onIdentityToken" type="primary" @click="saveToBrowser">
        {{ t('access.saveToBrowser') }}
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { Check, CopyDocument } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useMemoryConfig } from '../../config'
import { toastError, toastSuccess } from '../../toast'

defineProps<{
  /** In compact mode (<960px) the dialog widens to 96% */
  compact: boolean
}>()

const config = useMemoryConfig()

const visible = ref(false)
const title = ref('')
const identityName = ref('')
const token = ref('')

/** Show the one-time token (called by the parent after a successful create/reset). */
function show(newTitle: string, name: string, newToken: string): void {
  title.value = newTitle
  identityName.value = name
  token.value = newToken
  visible.value = true
}

defineExpose({ show })

// "Save to this browser": hand the one-time token to the host's token table (the standalone
// site shell remembers it and switches). The button only renders when the host injects
// onIdentityToken, so this is a non-null call.
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

// Copy keeps the dialog open (the token must stay visible until the user closes it
// themselves); the icon briefly flips to a checkmark as in-place feedback.
const justCopied = ref(false)
let copiedTimer: ReturnType<typeof setTimeout> | undefined

async function copyToken(value: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(value)
    justCopied.value = true
    clearTimeout(copiedTimer)
    copiedTimer = setTimeout(() => (justCopied.value = false), 1500)
    toastSuccess(t('access.copied'))
  } catch {
    toastError(value)
  }
}

onBeforeUnmount(() => clearTimeout(copiedTimer))
</script>

<style scoped>
.token-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 8px;
}
.new-token {
  flex: 1;
  padding: 10px 12px;
  border-radius: 6px;
  background: var(--el-fill-color);
  font-size: 13px;
  word-break: break-all;
  user-select: all;
}
</style>
