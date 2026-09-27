<template>
  <!-- 能力预设 + 勾选：身份表单与匿名访问卡共用；caps/preset 两个状态由父层持有（v-model 双向绑定） -->
  <div>
    <div class="presets">
      <span class="presets-label">{{ t('access.presets') }}</span>
      <el-radio-group v-model="preset" size="small" @change="applyPreset">
        <el-radio-button value="admin">{{ t('access.presetAdmin') }}</el-radio-button>
        <el-radio-button value="member">{{ t('access.presetMember') }}</el-radio-button>
        <el-radio-button value="viewer">{{ t('access.presetViewer') }}</el-radio-button>
        <el-radio-button value="custom">{{ t('access.presetCustom') }}</el-radio-button>
      </el-radio-group>
    </div>
    <div class="caps">
      <el-checkbox v-for="c in CAPS" :key="c.key" v-model="caps[c.key]" @change="onManualToggle">
        {{ capLabel(c.key) }}
      </el-checkbox>
    </div>
  </div>
</template>

<script setup lang="ts">
import { t } from '../../i18n'
import { CAPS, PRESETS, capLabel, type PresetKey } from './caps'

const caps = defineModel<Record<string, boolean>>('caps', { required: true })
const preset = defineModel<PresetKey>('preset', { required: true })

function applyPreset(): void {
  if (preset.value === 'custom') return
  const keys = new Set(PRESETS[preset.value] ?? [])
  for (const c of CAPS) caps.value[c.key] = keys.has(c.key)
}

function onManualToggle(): void {
  preset.value = 'custom'
}
</script>

<style scoped>
.presets {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
  flex-wrap: wrap;
}
.presets-label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.caps {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 16px;
}
</style>
