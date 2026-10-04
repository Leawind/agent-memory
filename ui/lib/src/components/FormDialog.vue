<template>
  <el-dialog
    :model-value="visible"
    :title="title"
    :width="width"
    :align-center="alignCenter"
    :append-to-body="appendToBody"
    :close-on-click-modal="!saving"
    :close-on-press-escape="!saving"
    :before-close="blockWhileSaving"
    :class="['am-form-dialog', { 'am-form-dialog--floating': appendToBody }]"
    @update:model-value="emit('update:visible', $event)"
  >
    <el-form label-position="top" @submit.prevent>
      <slot />
    </el-form>
    <template #footer>
      <slot name="footer">
        <el-button @click="emit('update:visible', false)">{{ t('common.cancel') }}</el-button>
        <el-button :type="submitType" :loading="saving" :disabled="submitDisabled" @click="emit('submit')">
          {{ submitText ?? t('common.save') }}
        </el-button>
      </slot>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { t } from '../i18n'

// Shared shell for create/edit/delete dialogs: dialog frame + top-labeled form + a
// cancel/submit footer. Business logic (fields, validation, the async submit) stays
// with the caller via the default slot and the submit event.
const props = withDefaults(
  defineProps<{
    /** Controlled visibility (v-model:visible) */
    visible: boolean
    title: string
    width?: string
    /** Loading state of the submit action; also blocks overlay/ESC/X closing while true */
    saving?: boolean
    /** Extra disabled state on the submit button (e.g. form still loading) */
    submitDisabled?: boolean
    /** Submit button label; defaults to the common "Save" */
    submitText?: string
    /** Submit button type; danger for destructive dialogs */
    submitType?: 'primary' | 'danger'
    alignCenter?: boolean
    appendToBody?: boolean
  }>(),
  {
    width: '480px',
    saving: false,
    submitDisabled: false,
    submitType: 'primary',
    alignCenter: false,
    appendToBody: false,
  },
)

const emit = defineEmits<{
  'update:visible': [value: boolean]
  submit: []
}>()

// before-close fires for the X button (overlay click / ESC are already off while saving)
function blockWhileSaving(done: () => void): void {
  if (!props.saving) done()
}
</script>

<!-- Unscoped: with append-to-body the dialog element lives outside this component tree -->
<style>
.am-form-dialog--floating {
  resize: both;
  overflow: auto;
  max-width: 95vw;
  max-height: 90vh;
}
.am-form-dialog--floating .el-dialog__body {
  /* At small viewport heights the min() shrinks the body so header and footer stay visible
     inside the 90vh dialog; at large heights the 65vh cap keeps the dialog from sprawling */
  max-height: min(65vh, calc(90vh - 150px));
  overflow-y: auto;
}
</style>
