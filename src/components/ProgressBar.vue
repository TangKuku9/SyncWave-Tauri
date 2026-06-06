<template>
  <div class="progress-bar-container" v-show="visible">
    <div class="progress-bar-bg">
      <div class="progress-bar-fill" :style="{ width: progress + '%' }"></div>
    </div>
    <div class="progress-text" v-if="text">{{ text }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

const props = defineProps<{
  progress: number
  text?: string
  visible?: boolean
}>()

const progress = ref(props.progress)
const text = ref(props.text)
const visible = ref(props.visible ?? true)

watch(() => props.progress, (v) => { progress.value = v })
watch(() => props.text, (v) => { text.value = v })
watch(() => props.visible, (v) => { visible.value = v ?? true })
</script>

<style scoped>
.progress-bar-container { flex-shrink: 0; }
.progress-bar-bg {
  height: 6px;
  background: var(--bg-tertiary);
  border-radius: 3px;
  overflow: hidden;
}
.progress-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--accent), #8b5cf6);
  border-radius: 3px;
  transition: width 0.3s ease;
}
.progress-text {
  font-size: 12px;
  color: var(--text-muted);
  margin-top: 6px;
  text-align: center;
}
</style>
