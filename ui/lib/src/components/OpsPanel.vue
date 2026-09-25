<template>
  <div ref="rootRef" class="am-panel">
    <div class="am-panel-header">
      <div>
        <h2 class="am-panel-title">运维</h2>
        <p class="am-panel-subtitle">数据库概况、备份导入导出与数据体检</p>
      </div>
      <el-button :icon="Refresh" @click="run(reload)">刷新概况</el-button>
    </div>

    <el-row :gutter="14">
      <el-col :span="compact ? 12 : 6">
        <el-card shadow="never">
          <el-statistic title="记忆条数" :value="stats.memories ?? 0" />
        </el-card>
      </el-col>
      <el-col :span="compact ? 12 : 6">
        <el-card shadow="never">
          <el-statistic title="标签数" :value="stats.tags ?? 0" />
        </el-card>
      </el-col>
      <el-col :span="compact ? 12 : 6">
        <el-card shadow="never">
          <el-statistic title="数据库大小" :value="sizeText" />
        </el-card>
      </el-col>
      <el-col :span="compact ? 12 : 6">
        <el-card shadow="never">
          <el-statistic title="下一个记忆 ID" :value="stats.next_id ?? '—'" />
        </el-card>
      </el-col>
    </el-row>

    <el-card shadow="never">
      <template #header>数据库</template>
      <el-descriptions :column="compact ? 1 : 2" border>
        <el-descriptions-item label="文件路径">{{ stats.path ?? '—' }}</el-descriptions-item>
        <el-descriptions-item label="最近更新">
          {{ stats.newest_update ? `${stats.newest_update.id}（${formatTime(stats.newest_update.updated_at)}）` : '—' }}
        </el-descriptions-item>
        <el-descriptions-item label="跨平台迁移">
          SQLite 文件格式平台无关，停服后可直接复制 .db 文件到其他机器；运行中请改用「导出备份」。
        </el-descriptions-item>
        <el-descriptions-item label="版本">{{ stats.version ?? version }}</el-descriptions-item>
      </el-descriptions>
      <div class="actions">
        <el-button :icon="Download" :loading="exporting" @click="run(exportData)">导出备份（JSON）</el-button>
        <el-button :icon="UploadFilled" :loading="importing" @click="importInput?.click()">导入备份</el-button>
        <input
          ref="importInput"
          type="file"
          accept="application/json,.json"
          style="display: none"
          @change="onImportFile"
        />
      </div>
      <el-alert
        title="导入仅支持空数据库（导入是恢复而非合并）；当前库已有数据时请换一个空的 --db 路径再导入。"
        type="info"
        show-icon
        :closable="false"
        class="import-hint"
      />
    </el-card>

    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <span>数据体检（doctor）</span>
          <el-button size="small" :icon="Search" :loading="doctorLoading" @click="run(runDoctor)">运行体检</el-button>
        </div>
      </template>
      <template v-if="doctorRan">
        <el-alert
          v-if="doctor.ok"
          title="体检通过：未发现孤儿引用、大小写冲突或空字段"
          type="success"
          show-icon
          :closable="false"
        />
        <el-alert v-else :title="`发现 ${doctor.issues.length} 个问题`" type="error" show-icon :closable="false" />
        <ul v-if="!doctor.ok" class="issues">
          <li v-for="(issue, i) in doctor.issues" :key="i">{{ issue }}</li>
        </ul>
      </template>
      <el-empty v-else description="点击「运行体检」检查孤儿标签引用、大小写冲突标签组、空摘要/正文" :image-size="60" />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { ElMessage } from 'element-plus'
import { Download, Refresh, Search, UploadFilled } from '@element-plus/icons-vue'
import { formatTime } from '../format'
import { useOps } from '../composables/useOps'
import { useContainerWidth } from '../composables/useContainerWidth'

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
const { compact } = useContainerWidth(rootRef)

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
      ElMessage.success(`已导入 ${imported.imported_memories} 条记忆、${imported.imported_tags} 个标签`)
    })
    .catch((err: unknown) => {
      ElMessage.error(err instanceof Error ? err.message : String(err))
    })
}
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
.import-hint {
  margin-top: 12px;
}
</style>
