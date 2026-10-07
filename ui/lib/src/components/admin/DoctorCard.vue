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

    <!-- Data at a glance: the parent already loads /api/stats, so this strip costs nothing extra -->
    <div v-if="stats" class="stat-strip">
      <span class="stat"
        >{{ t('ops.statMemories') }} <b>{{ stats.memories ?? 0 }}</b></span
      >
      <span class="stat"
        >{{ t('ops.statTags') }} <b>{{ stats.tags ?? 0 }}</b></span
      >
      <span class="stat"
        >{{ t('ops.statSize') }} <b>{{ sizeText }}</b></span
      >
      <span class="stat"
        >{{ t('ops.schemaVersion') }} <b>v{{ stats.schema_version ?? '—' }}</b></span
      >
    </div>

    <template v-if="doctorRan">
      <el-alert
        v-if="doctor.ok"
        :title="t('access.doctorOk', { total: doctor.checks.length })"
        type="success"
        show-icon
        :closable="false"
      />
      <el-alert
        v-else
        :title="
          t('access.doctorFail', {
            count: issueCount,
            failed: failedCount,
            total: doctor.checks.length,
          })
        "
        type="error"
        show-icon
        :closable="false"
      />

      <!-- Named checklist: every check gets a verdict row; failing ones list the raw issue lines -->
      <ul class="checks">
        <li v-for="check in doctor.checks" :key="check.id" :class="{ 'is-failed': !check.ok }">
          <div class="check-row">
            <el-icon class="verdict" :class="check.ok ? 'is-ok' : 'is-bad'">
              <CircleCheckFilled v-if="check.ok" />
              <CircleCloseFilled v-else />
            </el-icon>
            <span class="check-name">{{ checkName(check.id) }}</span>
            <span v-if="!check.ok" class="check-count">
              {{ t('access.doctorIssueCount', { count: check.issues.length }) }}
            </span>
          </div>
          <ul v-if="!check.ok" class="issues">
            <li v-for="(issue, i) in check.issues" :key="i">{{ issue }}</li>
          </ul>
        </li>
      </ul>
    </template>
    <!-- Before the first run, just one line of hint - no big placeholder block -->
    <div v-else class="pending">{{ t('access.doctorEmpty') }}</div>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { CircleCheckFilled, CircleCloseFilled, Search } from '@element-plus/icons-vue'
import { t } from '../../i18n'
import { useAdmin } from '../../composables/useAdmin'
import { formatSize } from '../../format'
import type { StatsInfo } from '../../types'
import { run } from './caps'

const props = defineProps<{
  /** Stats fetched by the parent's load: drives the at-a-glance strip (no extra request) */
  stats: StatsInfo | null
}>()

const { doctor, doctorRan, doctorLoading, runDoctor } = useAdmin()

const failedCount = computed(() => doctor.value.checks.filter((c) => !c.ok).length)
const issueCount = computed(() => doctor.value.checks.reduce((n, c) => n + c.issues.length, 0))
const sizeText = computed(() => formatSize(props.stats?.file_size))

// Server sends stable ids; the names are localized here. Unknown ids degrade to the raw id.
const CHECK_KEYS: Record<string, string> = {
  integrity: 'access.doctorCheckIntegrity',
  foreign_keys: 'access.doctorCheckForeignKeys',
  tag_refs: 'access.doctorCheckTagRefs',
  memory_refs: 'access.doctorCheckMemoryRefs',
  tag_case: 'access.doctorCheckTagCase',
  empty_summary: 'access.doctorCheckEmptySummary',
  timestamps: 'access.doctorCheckTimestamps',
  duplicates: 'access.doctorCheckDuplicates',
  orphan_embeddings: 'access.doctorCheckOrphanEmbeddings',
  embedding_coverage: 'access.doctorCheckEmbeddingCoverage',
}

function checkName(id: string): string {
  const key = CHECK_KEYS[id]
  return key ? t(key) : id
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.stat-strip {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 20px;
  margin-bottom: 14px;
}
.stat {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}
.stat b {
  color: var(--el-text-color-primary);
  font-variant-numeric: tabular-nums;
}
.checks {
  list-style: none;
  margin: 14px 0 0;
  padding: 0;
}
.check-row {
  display: flex;
  align-items: center;
  gap: 8px;
  line-height: 1.8;
}
.verdict.is-ok {
  color: var(--el-color-success);
}
.verdict.is-bad {
  color: var(--el-color-danger);
}
.check-name {
  font-size: 13px;
  color: var(--el-text-color-regular);
}
.is-failed .check-name {
  color: var(--el-text-color-primary);
}
.check-count {
  margin-left: auto;
  font-size: 12px;
  color: var(--el-color-danger);
}
.issues {
  margin: 2px 0 6px;
  padding-left: 30px;
  line-height: 1.8;
}
.pending {
  font-size: 12px;
  line-height: 1.6;
  color: var(--el-text-color-secondary);
}
</style>
