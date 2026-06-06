<template>
  <div class="toast-container" v-show="visible">
    <div class="toast" :class="type">
      <Icon :name="iconName" :size="16" />
      <span class="toast-text">{{ message }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import Icon from './Icon.vue'

const props = withDefaults(defineProps<{
  message?: string
  type?: 'success' | 'error' | 'warning' | 'info'
  duration?: number
}>(), { type: 'info', duration: 3000 })

const visible = ref(false)
let timer: number | undefined

watch(() => props.message, (val) => {
  if (val) {
    visible.value = true
    if (timer) clearTimeout(timer)
    timer = window.setTimeout(() => { visible.value = false }, props.duration)
  }
})

const iconName: Record<string, string> = {
  success: 'check', error: 'close', warning: 'warning', info: 'musicNote'
}
</script>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}
.toast {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  border-radius: var(--border-radius-md);
  font-size: 14px;
  font-weight: 500;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  animation: toastFadeIn 0.25s ease;
  white-space: nowrap;
}
.toast.success { background: #1a3a2a; color: var(--success); border: 1px solid rgba(74, 222, 128, 0.3); }
.toast.error { background: #3a1a1a; color: var(--error); border: 1px solid rgba(248, 113, 113, 0.3); }
.toast.warning { background: #3a2a1a; color: var(--warning); border: 1px solid rgba(251, 191, 36, 0.3); }
.toast.info { background: #1a1a3a; color: var(--accent); border: 1px solid rgba(108, 99, 255, 0.3); }
@keyframes toastFadeIn {
  from { opacity: 0; transform: translateY(12px); }
  to { opacity: 1; transform: translateY(0); }
}
</style>
