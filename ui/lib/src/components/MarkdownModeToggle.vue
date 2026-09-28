<template>
  <!-- Single-icon toggle: icon and tooltip point at the mode you get by clicking - in edit mode the eye lights up (go to preview), in preview mode the pencil lights up (back to edit) -->
  <el-tooltip :content="label" placement="top" :enterable="false">
    <el-button class="md-mode-toggle" size="small" :aria-label="label" @click="toggle">
      <el-icon>
        <View v-if="modelValue === 'edit'" />
        <Edit v-else />
      </el-icon>
    </el-button>
  </el-tooltip>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Edit, View } from '@element-plus/icons-vue'
import { t } from '../i18n'

const props = defineProps<{ modelValue: 'edit' | 'preview' }>()
const emit = defineEmits<{ 'update:modelValue': [value: 'edit' | 'preview'] }>()

const label = computed(() => (props.modelValue === 'edit' ? t('editor.tabPreview') : t('editor.tabEdit')))

function toggle(): void {
  emit('update:modelValue', props.modelValue === 'edit' ? 'preview' : 'edit')
}
</script>
