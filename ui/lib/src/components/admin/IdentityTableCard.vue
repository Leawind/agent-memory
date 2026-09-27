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
    <el-table v-else :data="identities">
      <el-table-column prop="name" :label="t('access.colName')" min-width="120" />
      <el-table-column :label="t('access.colToken')" min-width="200">
        <template #default="{ row }">
          <div class="token-cell">
            <code class="token-text">{{ maskToken(row.token_hint) }}</code>
            <el-button link type="primary" @click="emit('reset-token', row)">
              {{ t('access.resetToken') }}
            </el-button>
          </div>
        </template>
      </el-table-column>
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
      <el-table-column v-if="!compact" :label="t('access.colCreatedAt')" width="170">
        <template #default="{ row }">{{ formatTime(row.created_at) }}</template>
      </el-table-column>
      <el-table-column :label="t('access.colActions')" width="140" fixed="right">
        <template #default="{ row }">
          <el-button link type="primary" @click="emit('edit', row)">
            {{ t('common.edit') }}
          </el-button>
          <el-button link type="danger" @click="emit('delete', row)">
            {{ t('common.delete') }}
          </el-button>
        </template>
      </el-table-column>
    </el-table>
  </el-card>
</template>

<script setup lang="ts">
import { t } from '../../i18n'
import { formatTime } from '../../format'
import { Plus } from '@element-plus/icons-vue'
import { capLabel, enabledCaps, maskToken, type IdentityRow } from './caps'

defineProps<{
  identities: IdentityRow[]
  /** compact（<960px）时创建时间列收起 */
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
.token-cell {
  display: flex;
  align-items: center;
  gap: 8px;
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
</style>
