<template>
  <el-dialog
    :model-value="visible"
    :width="width ?? '560px'"
    align-center
    append-to-body
    :close-on-click-modal="!saving"
    :close-on-press-escape="!saving"
    :before-close="blockWhileSaving"
    class="am-form-dialog am-form-dialog--floating"
    @update:model-value="emit('update:visible', $event)"
  >
    <!-- Title row per the workspace spec: 作用域 #<name> + save. The name is the scope's
         identity: editable only at creation, locked afterwards (a rename would re-point every
         composed tag under the scope). Scopes carry no delete — dropping one is a store-level
         operation (its tags go with it). -->
    <template #header>
      <div class="am-dialog-head">
        <h3 class="am-dialog-title">
          {{ isCreate ? t('scopes.createTitle') : t('scopes.titleFmt', { name: original.name }) }}
        </h3>
        <div class="am-dialog-actions">
          <el-button type="primary" :loading="saving" :disabled="!canSave" @click="save">
            {{ t('common.save') }}
          </el-button>
        </div>
      </div>
      <div
        class="am-edge am-edge--left"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      />
      <div
        class="am-edge am-edge--right"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerUp"
      />
    </template>

    <el-form label-position="top" @submit.prevent>
      <el-form-item :label="t('scopes.nameLabel')">
        <el-input
          v-model="form.name"
          maxlength="100"
          show-word-limit
          :disabled="!isCreate"
          :placeholder="t('scopes.namePlaceholder')"
        />
        <div v-if="nameError" class="form-hint is-error">{{ nameError }}</div>
      </el-form-item>
      <el-form-item :label="t('scopes.descLabel')">
        <el-input
          v-model="form.description"
          type="textarea"
          :rows="4"
          maxlength="512"
          show-word-limit
          :placeholder="t('scopes.descPlaceholder')"
        />
      </el-form-item>
      <!-- A required scope is a write contract: every new memory must declare it — a value, or
           the explicit none. -->
      <el-form-item>
        <div class="required-row">
          <el-switch v-model="form.required" />
          <div class="required-copy">
            <span class="required-title">{{ t('scopes.requiredLabel') }}</span>
            <span class="required-hint">{{ t('scopes.requiredHint') }}</span>
          </div>
        </div>
      </el-form-item>
    </el-form>
  </el-dialog>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { toastError, toastSuccess } from '../toast'
import { useApiClient } from '../api/client'
import { createScope, updateScope } from '../api/tags'
import { validTagName } from '../tagNames'
import { t } from '../i18n'
import { useDialogEdgeResize } from '../composables/useDialogEdgeResize'
import type { ScopeView } from '../types'

const props = defineProps<{
  visible: boolean
  /** The scope being edited; null means creating new */
  scope: ScopeView | null
  /** Initial dialog width; defaults to 560px */
  width?: string
}>()

const { onPointerDown, onPointerMove, onPointerUp } = useDialogEdgeResize()

const emit = defineEmits<{
  'update:visible': [value: boolean]
  /** Saved successfully (create or update finished) */
  saved: []
}>()

const client = useApiClient()

const isCreate = computed(() => props.scope === null)

const saving = ref(false)
const form = reactive({ name: '', description: '', required: false })
// Snapshot of the opened scope: identity for the title row, baseline for the dirty check
const original = reactive({ name: '', description: '', required: false })

const dirty = computed(
  () =>
    form.name.trim() !== original.name ||
    form.description !== original.description ||
    form.required !== original.required,
)
// Illegal names are flagged as they are typed, not on the server round-trip; a scope name is a
// single identifier word — `::` belongs to the tag names pointing into it
const nameError = computed(() =>
  form.name.trim() !== '' && !validTagName(form.name) ? t('scopes.invalidNameFmt', { name: form.name.trim() }) : '',
)
const canSave = computed(() => !saving.value && !nameError.value && form.name.trim() !== '' && dirty.value)

watch(
  () => props.visible,
  (open) => {
    if (!open) return
    original.name = props.scope?.name ?? ''
    original.description = props.scope?.description ?? ''
    original.required = props.scope?.required === true
    form.name = props.scope?.name ?? ''
    form.description = props.scope?.description ?? ''
    form.required = props.scope?.required === true
  },
)

async function save() {
  if (!canSave.value) return
  saving.value = true
  try {
    if (isCreate.value) {
      await createScope(client, form.name.trim(), form.description, form.required ? true : undefined)
      toastSuccess(t('scopes.created'))
    } else {
      // The name is the identity and never travels; the flag goes only when it actually
      // changed, so a description edit never re-arms the contract
      const flag = form.required !== original.required ? form.required : undefined
      await updateScope(client, original.name, form.description, flag)
      toastSuccess(t('scopes.saved'))
    }
    emit('update:visible', false)
    emit('saved')
  } catch (e: unknown) {
    toastError(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}

// before-close fires for the X button (overlay click / ESC are already off while saving)
function blockWhileSaving(done: () => void): void {
  if (!saving.value) done()
}
</script>

<style scoped>
.required-row {
  display: flex;
  align-items: flex-start;
  gap: 10px;
}
.required-copy {
  display: flex;
  flex-direction: column;
  line-height: 1.4;
}
.required-hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.form-hint {
  font-size: 12px;
  line-height: 1.5;
  color: var(--el-text-color-secondary);
  width: 100%;
  margin-top: 4px;
}
.form-hint.is-error {
  color: var(--el-color-danger);
}
</style>
