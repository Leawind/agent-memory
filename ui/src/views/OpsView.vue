<template>
  <div class="page">
    <el-row :gutter="14">
      <el-col :span="6">
        <el-card shadow="never">
          <el-statistic title="记忆条数" :value="stats.memories ?? 0" />
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="never">
          <el-statistic title="标签数" :value="stats.tags ?? 0" />
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="never">
          <el-statistic title="数据库大小" :value="sizeText" />
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="never">
          <el-statistic title="下一个记忆 ID" :value="stats.next_id ?? '—'" />
        </el-card>
      </el-col>
    </el-row>

    <el-card shadow="never">
      <template #header>数据库</template>
      <el-descriptions :column="2" border>
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
        <el-button :icon="Download" @click="exportData" :loading="exporting">导出备份（JSON）</el-button>
        <el-button :icon="UploadFilled" @click="importInput?.click()" :loading="importing">导入备份</el-button>
        <el-button :icon="Refresh" @click="reload">刷新概况</el-button>
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
          <el-button size="small" :icon="Search" :loading="doctorLoading" @click="runDoctor">运行体检</el-button>
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
import { computed, onMounted, ref } from 'vue'
import { ElMessage } from 'element-plus'
import { Download, Refresh, Search, UploadFilled } from '@element-plus/icons-vue'
import { get, formatTime, formatSize } from '../api'
import type { DoctorResp, HealthInfo, ImportResp, StatsInfo } from '../types'

const stats = ref<Partial<StatsInfo>>({})
const doctor = ref<DoctorResp>({ ok: true, issues: [] })
const doctorRan = ref(false)
const doctorLoading = ref(false)
const exporting = ref(false)
const importing = ref(false)
const importInput = ref<HTMLInputElement | null>(null)
const version = ref('')

const sizeText = computed(() => formatSize(stats.value.file_size))

async function reload() {
  try {
    stats.value = await get<StatsInfo>('/api/stats')
    stats.value.version = version.value
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  }
}

async function onImportFile(e: Event) {
  const input = e.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  importing.value = true
  try {
    const text = await file.text()
    let dump: unknown
    try {
      dump = JSON.parse(text)
    } catch {
      throw new Error('备份文件不是有效的 JSON')
    }
    const res = await fetch('/api/import', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(dump),
    })
    const data = (await res.json().catch(() => null)) as ImportResp | { error?: string } | null
    if (!res.ok) {
      throw new Error((data as { error?: string })?.error ?? `导入失败 (HTTP ${res.status})`)
    }
    ElMessage.success(
      `已导入 ${(data as ImportResp).imported_memories} 条记忆、${(data as ImportResp).imported_tags} 个标签`,
    )
    await reload()
  } catch (err: unknown) {
    ElMessage.error(err instanceof Error ? err.message : String(err))
  } finally {
    importing.value = false
  }
}

async function runDoctor() {
  doctorLoading.value = true
  try {
    doctor.value = await get<DoctorResp>('/api/doctor')
    doctorRan.value = true
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  } finally {
    doctorLoading.value = false
  }
}

async function exportData() {
  exporting.value = true
  try {
    const res = await fetch('/api/export')
    if (!res.ok) throw new Error(`导出失败 (HTTP ${res.status})`)
    const blob = await res.blob()
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'agent-memory-export.json'
    a.click()
    URL.revokeObjectURL(url)
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  } finally {
    exporting.value = false
  }
}

onMounted(async () => {
  try {
    version.value = (await get<HealthInfo>('/health')).version ?? ''
  } catch {
    /* ignore */
  }
  reload()
})
</script>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.actions {
  margin-top: 14px;
  display: flex;
  gap: 10px;
}
.issues {
  margin: 12px 0 0;
  padding-left: 20px;
  line-height: 1.9;
}
</style>
