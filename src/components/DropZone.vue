<template>
  <div
    class="drop-zone"
    :class="{ 'drag-over': isDragOver }"
    @dragover.prevent="isDragOver = true"
    @dragleave="isDragOver = false"
    @drop.prevent="onDrop"
    @click="onSelect"
  >
    <div class="drop-zone-icon">
      <Icon :name="iconName" :size="48" />
    </div>
    <div class="drop-zone-text">{{ text }}</div>
    <div class="drop-zone-hint">{{ hint }}</div>
    <button class="drop-zone-btn" @click.stop="onSelect">{{ buttonText }}</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Icon from './Icon.vue'

const props = defineProps<{
  iconName?: string
  text?: string
  hint?: string
  buttonText?: string
}>()

const emit = defineEmits<{
  select: []
  drop: [files: string[]]
}>()

const isDragOver = ref(false)

function onDrop(e: DragEvent) {
  isDragOver.value = false
  // Note: In Tauri, drag-drop files from OS works differently
  // The actual file paths come from Tauri's drag-drop event
  emit('select')
}

function onSelect() {
  emit('select')
}
</script>

<style scoped>
.drop-zone {
  border: 2px dashed var(--border);
  border-radius: var(--border-radius-xl);
  padding: 32px;
  text-align: center;
  transition: all var(--transition-slow);
  cursor: pointer;
  background: var(--bg-card);
  flex-shrink: 0;
}
.drop-zone:hover,
.drop-zone.drag-over {
  border-color: var(--accent);
  background: rgba(108, 99, 255, 0.08);
  box-shadow: 0 0 30px var(--accent-glow);
}
.drop-zone.drag-over { transform: scale(1.01); }
.drop-zone-icon {
  margin-bottom: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent);
}
.drop-zone-text { font-size: 16px; color: var(--text-secondary); margin-bottom: 6px; }
.drop-zone-hint { font-size: 13px; color: var(--text-muted); }
.drop-zone-btn {
  display: inline-block;
  margin-top: 16px;
  padding: 10px 24px;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: var(--border-radius-md);
  font-size: 14px;
  cursor: pointer;
  transition: all var(--transition-normal);
  font-weight: 500;
}
.drop-zone-btn:hover {
  background: var(--accent-hover);
  box-shadow: 0 4px 15px var(--accent-glow);
}
</style>
