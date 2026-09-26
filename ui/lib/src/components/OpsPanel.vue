<template>
  <div ref="rootRef" class="am-panel">
    <div v-if="showHeader" class="am-panel-header">
      <div class="am-heading">
        <h2 class="am-panel-title">{{ props.title ?? t('ops.title') }}</h2>
        <el-tooltip :content="props.subtitle ?? t('ops.subtitle')" placement="top">
          <el-icon class="am-info"><InfoFilled /></el-icon>
        </el-tooltip>
      </div>
      <el-button :icon="Refresh" @click="run(reload)">{{ t('ops.refresh') }}</el-button>
    </div>

    <el-row :gutter="14">
      <el-col :span="narrow ? 12 : 8">
        <el-card shadow="never">
          <el-statistic :title="t('ops.statMemories')" :value="stats.memories ?? 0" />
        </el-card>
      </el-col>
      <el-col :span="narrow ? 12 : 8">
        <el-card shadow="never">
          <el-statistic :title="t('ops.statTags')" :value="stats.tags ?? 0" />
        </el-card>
      </el-col>
      <el-col :span="narrow ? 12 : 8">
        <el-card shadow="never">
          <el-statistic :title="t('ops.statSize')" :value="sizeText" />
        </el-card>
      </el-col>
    </el-row>

    <el-card shadow="never">
      <template #header>{{ t('ops.dbCard') }}</template>
      <!-- 版本与数据库版本同行；文件路径较长独占一行 -->
      <el-descriptions :column="narrow ? 1 : 2" border>
        <el-descriptions-item :label="t('ops.version')">{{ stats.version ?? version }}</el-descriptions-item>
        <el-descriptions-item :label="t('ops.schemaVersion')">
          {{ stats.schema_version ?? '—' }}
        </el-descriptions-item>
        <el-descriptions-item :label="t('ops.path')" :span="narrow ? 1 : 2">
          {{ stats.path ?? '—' }}
        </el-descriptions-item>
      </el-descriptions>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { InfoFilled, Refresh } from '@element-plus/icons-vue'
import { t } from '../i18n'
import { useOps } from '../composables/useOps'
import { useContainerWidth } from '../composables/useContainerWidth'
import { toastError } from '../toast'

const props = withDefaults(
  defineProps<{
    /** 隐藏标题/副标题区（嵌入宿主已有页面标题时只要内容卡） */
    showHeader?: boolean
    /** 覆盖默认标题 */
    title?: string
    /** 覆盖默认副标题 */
    subtitle?: string
  }>(),
  { showHeader: true },
)

const { stats, version, sizeText, reload } = useOps()

const rootRef = ref<HTMLElement | null>(null)
// narrow（<720px）时统计卡两列排布、数据库信息单列
const { narrow } = useContainerWidth(rootRef)

function run(action: () => Promise<unknown>) {
  return action().catch((e: unknown) => toastError(e instanceof Error ? e.message : String(e)))
}

// 面板常驻挂载时无法自行感知可见性：宿主切回此面板时调 refresh 拉最新数据
defineExpose({ refresh: () => run(reload) })
</script>
