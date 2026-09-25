<template>
  <div class="page">
    <div class="toolbar">
      <el-input
        v-model="query"
        placeholder="关键词搜索（空格分隔、全部命中；中文按子串匹配）"
        clearable
        class="search"
        @keyup.enter="onSearch"
        @clear="onSearch"
      >
        <template #append>
          <el-button :icon="Search" @click="onSearch" />
        </template>
      </el-input>
      <el-select
        v-model="tagFilter"
        placeholder="按标签过滤"
        clearable
        filterable
        class="tag-filter"
        @change="onSearch"
      >
        <el-option v-for="t in tagOptions" :key="t" :label="t" :value="t" />
      </el-select>
      <el-select v-model="sort" class="sort" @change="reload">
        <el-option label="按更新时间" value="updated_at" />
        <el-option label="按创建时间" value="created_at" />
      </el-select>
      <el-select v-model="order" class="order" @change="reload">
        <el-option label="倒序" value="desc" />
        <el-option label="正序" value="asc" />
      </el-select>
      <div class="spacer" />
      <el-button type="primary" :icon="Plus" @click="openCreate">新建记忆</el-button>
    </div>

    <el-alert v-if="note" :title="note" type="info" show-icon :closable="false" class="note" />

    <!-- 搜索模式：显示匹配片段与评分 -->
    <el-table v-if="searching" :data="searchResults" v-loading="loading" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column label="摘要">
        <template #default="{ row }">
          <div class="summary">{{ row.summary }}</div>
          <div class="snippet" v-html="row.snippet" />
        </template>
      </el-table-column>
      <el-table-column label="标签" width="220">
        <template #default="{ row }">
          <el-tag v-for="t in row.tags" :key="t" size="small" class="tag">{{ t }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="score" label="评分" width="80" sortable />
      <el-table-column label="更新时间" width="170">
        <template #default="{ row }">{{ formatTime(row.updated_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="160" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" @click="openDetail(row.id)">详情</el-button>
          <el-button link type="primary" @click="openEdit(row.id)">编辑</el-button>
          <el-button link type="danger" @click="remove(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <!-- 列表模式 -->
    <el-table v-else :data="rows" v-loading="loading" stripe>
      <el-table-column prop="id" label="ID" width="80" />
      <el-table-column prop="summary" label="摘要" min-width="300" show-overflow-tooltip />
      <el-table-column label="标签" width="220">
        <template #default="{ row }">
          <el-tag v-for="t in row.tags" :key="t" size="small" class="tag">{{ t }}</el-tag>
        </template>
      </el-table-column>
      <el-table-column label="创建时间" width="170">
        <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column label="更新时间" width="170">
        <template #default="{ row }">{{ formatTime(row.updated_at) }}</template>
      </el-table-column>
      <el-table-column label="操作" width="160" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" @click="openDetail(row.id)">详情</el-button>
          <el-button link type="primary" @click="openEdit(row.id)">编辑</el-button>
          <el-button link type="danger" @click="remove(row)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <div class="pager" v-if="!searching">
      <el-pagination
        v-model:current-page="page"
        v-model:page-size="pageSize"
        :total="total"
        :page-sizes="[20, 50, 100, 200]"
        layout="total, sizes, prev, pager, next"
        @current-change="reload"
        @size-change="reload"
      />
    </div>
    <div class="pager" v-else>
      <el-pagination
        v-model:current-page="page"
        v-model:page-size="pageSize"
        :total="total"
        :page-sizes="[10, 20, 50]"
        layout="total, sizes, prev, pager, next"
        @current-change="reload"
        @size-change="reload"
      />
    </div>

    <!-- 新建 / 编辑 -->
    <el-dialog v-model="dialogVisible" :title="form.id ? '编辑记忆' : '新建记忆'" width="640px">
      <el-form label-position="top">
        <el-form-item label="摘要（列表与搜索展示的一行简介）">
          <el-input v-model="form.summary" maxlength="512" show-word-limit placeholder="精确、自洽的一句话" />
        </el-form-item>
        <el-form-item label="正文（完整内容）">
          <el-input v-model="form.content" type="textarea" :rows="10" maxlength="200000" show-word-limit />
        </el-form-item>
        <el-form-item label="标签（回车添加，可新建）">
          <el-select
            v-model="form.tags"
            multiple
            filterable
            allow-create
            default-first-option
            placeholder="选择或输入标签"
            class="tags-select"
          >
            <el-option v-for="t in tagOptions" :key="t" :label="t" :value="t" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="saving" @click="save">保存</el-button>
      </template>
    </el-dialog>

    <!-- 全文详情 -->
    <el-drawer v-model="detailVisible" :title="`记忆 ${detail?.id ?? ''}`" size="45%">
      <template v-if="detail">
        <h3 class="detail-summary">{{ detail.summary }}</h3>
        <div class="detail-tags">
          <el-tag v-for="t in detail.tags" :key="t" size="small" class="tag">{{ t }}</el-tag>
        </div>
        <el-divider />
        <pre class="detail-content">{{ detail.content }}</pre>
      </template>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search, Plus } from '@element-plus/icons-vue'
import { get, post, put, del, formatTime } from '../api'
import { buildMemoriesQuery, isSearchMode } from '../query'
import type { MemoryFull, MemoryListResp, MemorySearchResp, MemorySummary, SearchResult, TagListResp } from '../types'

interface MemoryForm {
  id: string | null
  summary: string
  content: string
  tags: string[]
}

const query = ref('')
const tagFilter = ref('')
const sort = ref<'updated_at' | 'created_at'>('updated_at')
const order = ref<'asc' | 'desc'>('desc')
const page = ref(1)
const pageSize = ref(20)
const rows = ref<MemorySummary[]>([])
const searchResults = ref<SearchResult[]>([])
const total = ref(0)
const note = ref('')
const loading = ref(false)
const tagOptions = ref<string[]>([])

const dialogVisible = ref(false)
const saving = ref(false)
const form = ref<MemoryForm>({ id: null, summary: '', content: '', tags: [] })
const detailVisible = ref(false)
const detail = ref<MemoryFull | null>(null)

const searching = computed(() => isSearchMode(query.value))

// 搜索词/过滤标签变化属于新的查询意图：从第一页重新开始
function onSearch() {
  page.value = 1
  reload()
}

function buildQuery() {
  return buildMemoriesQuery({
    query: query.value,
    tagFilter: tagFilter.value,
    sort: sort.value,
    order: order.value,
    page: page.value,
    pageSize: pageSize.value,
  })
}

// 请求序号：连续触发查询时（输入回车/清空/翻页）丢弃迟到的过期响应，
// 避免慢的旧结果覆盖新结果
let requestSeq = 0

async function reload() {
  const seq = ++requestSeq
  loading.value = true
  try {
    const qs = buildQuery()
    if (isSearchMode(query.value)) {
      const data = await get<MemorySearchResp>(`/api/memories?${qs}`)
      if (seq !== requestSeq) return
      searchResults.value = data.results ?? []
      total.value = data.total_matches ?? 0
      note.value = ''
    } else {
      let data = await get<MemoryListResp>(`/api/memories?${qs}`)
      if (seq !== requestSeq) return
      // 删除/过滤后当前页可能超出范围：回退到最后一页重新拉取
      const totalPages = Math.max(1, Math.ceil(data.total / pageSize.value))
      if (data.memories?.length === 0 && data.total > 0 && page.value > totalPages) {
        page.value = totalPages
        loading.value = false
        return reload()
      }
      rows.value = data.memories ?? []
      total.value = data.total ?? 0
      note.value = data.note ?? ''
    }
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  } finally {
    if (seq === requestSeq) loading.value = false
  }
}

async function loadTagOptions() {
  try {
    const data = await get<TagListResp>('/api/tags')
    tagOptions.value = (data.tags ?? []).map((t) => t.name)
  } catch {
    /* 静默：标签下拉失败不阻塞主列表 */
  }
}

function openCreate() {
  form.value = { id: null, summary: '', content: '', tags: [] }
  dialogVisible.value = true
}

async function openEdit(id: string) {
  try {
    const full = await get<MemoryFull>(`/api/memories/${id}`)
    form.value = { id, summary: full.summary, content: full.content, tags: [...full.tags] }
    dialogVisible.value = true
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  }
}

async function openDetail(id: string) {
  try {
    detail.value = await get<MemoryFull>(`/api/memories/${id}`)
    detailVisible.value = true
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  }
}

async function save() {
  saving.value = true
  try {
    if (form.value.id) {
      // 编辑：以当前标签为目标集，换算成 add_tags / remove_tags
      const current = await get<MemoryFull>(`/api/memories/${form.value.id}`)
      const before = new Set<string>(current.tags)
      const after = new Set<string>(form.value.tags)
      const add = [...after].filter((t) => !before.has(t))
      const remove = [...before].filter((t) => !after.has(t))
      await put(`/api/memories/${form.value.id}`, {
        summary: form.value.summary,
        content: form.value.content,
        add_tags: add,
        remove_tags: remove,
      })
      ElMessage.success('已更新')
    } else {
      await post('/api/memories', {
        summary: form.value.summary,
        content: form.value.content,
        tags: form.value.tags,
      })
      ElMessage.success('已创建')
    }
    dialogVisible.value = false
    await Promise.all([reload(), loadTagOptions()])
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  } finally {
    saving.value = false
  }
}

async function remove(row: MemorySummary) {
  try {
    await ElMessageBox.confirm(`确定永久删除记忆 ${row.id}？`, '删除确认', { type: 'warning' })
  } catch {
    return
  }
  try {
    await del(`/api/memories/${row.id}`)
    ElMessage.success('已删除')
    await reload()
  } catch (e: unknown) {
    ElMessage.error(e instanceof Error ? e.message : String(e))
  }
}

onMounted(() => {
  reload()
  loadTagOptions()
})
</script>

<style scoped>
.page {
  display: flex;
  flex-direction: column;
  gap: 14px;
}
.toolbar {
  display: flex;
  gap: 10px;
  align-items: center;
}
.search {
  width: 360px;
}
.tag-filter {
  width: 160px;
}
.sort,
.order {
  width: 130px;
}
.spacer {
  flex: 1;
}
.note {
  margin: 0;
}
.tag {
  margin-right: 4px;
}
.pager {
  display: flex;
  justify-content: flex-end;
}
.summary {
  font-weight: 500;
}
.snippet {
  color: var(--el-text-color-secondary);
  font-size: 12px;
  margin-top: 4px;
}
.tags-select {
  width: 100%;
}
.detail-summary {
  margin: 0 0 10px;
}
.detail-tags {
  margin-bottom: 6px;
}
.detail-content {
  white-space: pre-wrap;
  word-break: break-word;
  font-family: inherit;
  line-height: 1.7;
  margin: 0;
}
</style>
