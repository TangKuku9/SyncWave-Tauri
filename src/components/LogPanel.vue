<template>
  <div class="log-section">
    <div class="section-header">
      <span class="section-title">{{ title }}</span>
    </div>
    <div class="log-container" ref="logRef">
      <div class="log-entry" v-if="logs.length === 0">
        <span class="log-time">[就绪]</span>
        <span>等待任务...</span>
      </div>
      <div class="log-entry" v-for="(log, i) in logs" :key="i">
        <span class="log-time">[{{ log.time }}]</span>
        <span :class="log.type === 'success' ? 'log-success' : log.type === 'error' ? 'log-error' : ''">{{ log.message }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'

export interface LogEntry {
  time: string
  message: string
  type: string
}

const props = defineProps<{
  title: string
  logs: LogEntry[]
}>()

const logRef = ref<HTMLElement>()

watch(() => props.logs.length, async () => {
  await nextTick()
  if (logRef.value) logRef.value.scrollTop = logRef.value.scrollHeight
})
</script>

<style scoped>
.log-section { flex-shrink: 0; }
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}
.section-title { font-size: 14px; font-weight: 600; color: var(--text-secondary); }
.log-container {
  max-height: 150px;
  overflow-y: auto;
  background: var(--bg-card);
  border-radius: var(--border-radius-lg);
  border: 1px solid var(--border);
  padding: 8px 12px;
  font-family: 'Cascadia Code', 'Consolas', monospace;
  font-size: 12px;
}
.log-entry { padding: 3px 0; display: flex; gap: 8px; }
.log-time { color: var(--text-muted); flex-shrink: 0; }
.log-success { color: var(--success); }
.log-error { color: var(--error); }
</style>
