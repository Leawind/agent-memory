<template>
  <el-card shadow="never">
    <!-- 新建入口紧贴身份表（卡片头部右侧），开放模式警告与鉴权开关留在其原有语义位置 -->
    <template #header>
      <div class="card-header">
        <span>{{ t('access.identityCard') }}</span>
        <el-button type="primary" :icon="Plus" @click="emit('create')">
          {{ t('access.create') }}
        </el-button>
      </div>
    </template>
    <el-empty v-if="identities.length === 0" :description="t('access.empty')" />
    <!-- 窄容器：卡片列表替代表格——el-table 的固定操作列在手机宽度会把能力标签挤碎 -->
    <div v-else-if="compact" class="id-cards">
      <div v-for="row in identities" :key="row.name" class="id-card">
        <div class="id-card-head">
          <span class="id-card-name">{{ row.name }}</span>
          <span class="id-card-actions">
            <el-tooltip :content="t('common.edit')" placement="top">
              <el-button link type="primary" :icon="Edit" :aria-label="t('common.edit')" @click="emit('edit', row)" />
            </el-tooltip>
            <el-tooltip :content="t('access.resetToken')" placement="top">
              <el-button
                link
                type="primary"
                :icon="RefreshRight"
                :aria-label="t('access.resetToken')"
                @click="emit('reset-token', row)"
              />
            </el-tooltip>
            <el-tooltip :content="t('common.delete')" placement="top">
              <el-button
                link
                type="danger"
                :icon="Delete"
                :aria-label="t('common.delete')"
                @click="emit('delete', row)"
              />
            </el-tooltip>
          </span>
        </div>
        <div class="id-card-caps">
          <el-tag
            v-for="c in enabledCaps(row)"
            :key="c"
            size="small"
            class="cap-tag"
            :type="c === 'admin' ? 'danger' : 'info'"
          >
            {{ capLabel(c) }}
          </el-tag>
          <span v-if="enabledCaps(row).length === 0" class="muted">—</span>
        </div>
        <code class="token-text">{{ maskToken(row.token_hint) }}</code>
      </div>
    </div>
    <el-table v-else :data="identities">
      <el-table-column prop="name" :label="t('access.colName')" min-width="120" />
      <el-table-column :label="t('access.colPermissions')" min-width="240">
        <template #default="{ row }">
          <el-tag
            v-for="c in enabledCaps(row)"
            :key="c"
            size="small"
            class="cap-tag"
            :type="c === 'admin' ? 'danger' : 'info'"
          >
            {{ capLabel(c) }}
          </el-tag>
          <span v-if="enabledCaps(row).length === 0" class="muted">—</span>
        </template>
      </el-table-column>
      <el-table-column :label="t('access.colCreatedAt')" width="170">
        <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column :label="t('access.colToken')" min-width="200">
        <template #default="{ row }">
          <code class="token-text">{{ maskToken(row.token_hint) }}</code>
        </template>
      </el-table-column>
      <el-table-column :label="t('access.colActions')" width="130" fixed="right">
        <template #default="{ row }">
          <el-tooltip :content="t('common.edit')" placement="top">
            <el-button link type="primary" :icon="Edit" :aria-label="t('common.edit')" @click="emit('edit', row)" />
          </el-tooltip>
          <el-tooltip :content="t('access.resetToken')" placement="top">
            <el-button
              link
              type="primary"
              :icon="RefreshRight"
              :aria-label="t('access.resetToken')"
              @click="emit('reset-token', row)"
            />
          </el-tooltip>
          <el-tooltip :content="t('common.delete')" placement="top">
            <el-button
              link
              type="danger"
              :icon="Delete"
              :aria-label="t('common.delete')"
              @click="emit('delete', row)"
            />
          </el-tooltip>
        </template>
      </el-table-column>
    </el-table>
  </el-card>
</template>

<script setup lang="ts">
import { t } from '../../i18n'
import { formatTime } from '../../format'
import { Delete, Edit, Plus, RefreshRight } from '@element-plus/icons-vue'
import { capLabel, enabledCaps, maskToken, type IdentityRow } from './caps'

defineProps<{
  identities: IdentityRow[]
  /** compact（<960px）时以卡片列表替代表格（创建时间/token 列随之收起） */
  compact: boolean
}>()

const emit = defineEmits<{
  create: []
  edit: [row: IdentityRow]
  delete: [row: IdentityRow]
  'reset-token': [row: IdentityRow]
}>()
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.token-text {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.cap-tag {
  margin-right: 4px;
}
.muted {
  color: var(--el-text-color-secondary);
}
/* 窄容器的身份卡片列表 */
.id-cards {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.id-card {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 12px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: var(--el-border-radius-base);
}
.id-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.id-card-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 600;
  color: var(--el-text-color-primary);
}
.id-card-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}
.id-card-caps {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}
.id-card-caps .cap-tag {
  margin-right: 0;
}
</style>
