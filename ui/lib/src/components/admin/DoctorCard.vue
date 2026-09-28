<template>
  <el-card shadow="never">
    <template #header>
      <div class="card-header">
        <span>{{ t('access.doctorCard') }}</span>
        <el-button size="small" :icon="Search" :loading="doctorLoading" @click="run(runDoctor)">
          {{ t('access.runDoctor') }}
        </el-button>
      </div>
    </template>
    <template v-if="doctorRan">
      <el-alert v-if="doctor.ok" :title="t('access.doctorOk')" type="success" show-icon :closable="false" />
      <el-alert
        v-else
        :title="t('access.doctorFail', { count: doctor.issues.length })"
        type="error"
        show-icon
        :closable="false"
      />
      <ul v-if="!doctor.ok" class="issues">
        <li v-for="(issue, i) in doctor.issues" :key="i">{{ issue }}</li>
      </ul>
    </template>
    <!-- 未运行只留一行说明，不占大块空间 -->
    <div v-else class="pending">{{ t('access.doctorEmpty') }}</div>
  </el-card>
</template>

<script setup lang="ts">
import { Search } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useAdmin } from '../../composables/useAdmin'
import { run } from './caps'

const { doctor, doctorRan, doctorLoading, runDoctor } = useAdmin()
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.issues {
  margin: 12px 0 0;
  padding-left: 20px;
  line-height: 1.9;
}
.pending {
  font-size: 12px;
  line-height: 1.6;
  color: var(--el-text-color-secondary);
}
</style>
