<template>
  <!-- 单图标切换：图标与提示都指向点击后的模式——编辑态亮眼睛进预览，预览态亮铅笔回编辑 -->
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
