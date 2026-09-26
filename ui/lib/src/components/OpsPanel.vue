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
      <div class="actions">
        <el-button :icon="Download" :loading="exporting" @click="run(exportData)">{{ t('ops.export') }}</el-button>
        <el-button :icon="UploadFilled" :loading="importing" @click="importInput?.click()">{{
          t('ops.import')
        }}</el-button>
        <el-tooltip :content="t('ops.importHint')" placement="top">
          <el-icon class="am-info"><InfoFilled /></el-icon>
        </el-tooltip>
        <input
          ref="importInput"
          type="file"
          accept="application/json,.json"
          style="display: none"
          @change="onImportFile"
        />
      </div>
    </el-card>

    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <span>{{ t('ops.doctorCard') }}</span>
          <el-button size="small" :icon="Search" :loading="doctorLoading" @click="run(runDoctor)">
            {{ t('ops.runDoctor') }}
          </el-button>
        </div>
      </template>
      <template v-if="doctorRan">
        <el-alert v-if="doctor.ok" :title="t('ops.doctorOk')" type="success" show-icon :closable="false" />
        <el-alert
          v-else
          :title="t('ops.doctorFail', { count: doctor.issues.length })"
          type="error"
          show-icon
          :closable="false"
        />
        <ul v-if="!doctor.ok" class="issues">
          <li v-for="(issue, i) in doctor.issues" :key="i">{{ issue }}</li>
        </ul>
      </template>
      <el-empty v-else :description="t('ops.doctorEmpty')" :image-size="60" />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { Download, InfoFilled, Refresh, Search, UploadFilled } from '@element-plus/icons-vue'
import { t } from '../i18n'
import { useOps } from '../composables/useOps'
import { useContainerWidth } from '../composables/useContainerWidth'

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

const {
  stats,
  version,
  doctor,
  doctorRan,
  doctorLoading,
  exporting,
  importing,
  sizeText,
  reload,
  runDoctor,
  exportData,
  importFile,
} = useOps()

const rootRef = ref<HTMLElement | null>(null)
// narrow（<720px）时统计卡两列排布、数据库信息单列
const { narrow } = useContainerWidth(rootRef)

const importInput = ref<HTMLInputElement | null>(null)

function run(action: () => Promise<unknown>) {
  return action().catch((e: unknown) => ElMessage.error(e instanceof Error ? e.message : String(e)))
}

function onImportFile(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  importFile(file)
    .then((imported) => {
      ElMessage.success(t('ops.imported', { memories: imported.imported_memories, tags: imported.imported_tags }))
    })
    .catch((err: unknown) => {
      ElMessage.error(err instanceof Error ? err.message : String(err))
    })
}

// 面板常驻挂载时无法自行感知可见性：宿主切回此面板时调 refresh 拉最新数据
defineExpose({ refresh: () => run(reload) })
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.actions {
  margin-top: 14px;
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}
.issues {
  margin: 12px 0 0;
  padding-left: 20px;
  line-height: 1.9;
}
.label-help {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
</style>
