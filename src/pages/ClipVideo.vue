<template>
  <div class="page-container active clip-video-page">
    <div class="content-header"><h2>视频截取</h2><span class="badge">ffmpeg</span></div>
    <div class="file-select-container">
      <div class="file-select-box" :class="{ 'has-file': filePath }" @click="selectFile">
        <Icon name="videoCamera" :size="40" />
        <div class="file-select-label">选择视频文件</div>
        <div class="file-select-hint">点击选择要截取的视频</div>
        <div class="file-select-info" v-if="fileName">{{ fileName }}</div>
        <button class="file-select-btn">选择视频</button>
      </div>
    </div>
    <div class="dir-settings">
      <div class="dir-group">
        <div class="dir-label">开始时间</div>
        <input type="text" class="dir-input" v-model="startTime" placeholder="00:00:00 或 秒数" />
      </div>
      <div class="dir-group">
        <div class="dir-label">结束时间</div>
        <input type="text" class="dir-input" v-model="endTime" placeholder="00:00:00 或 秒数" />
      </div>
      <div class="dir-group">
        <div class="dir-label">输出目录</div>
        <div class="dir-input-row">
          <input type="text" class="dir-input" v-model="outputDir" placeholder="默认：原文件所在目录" />
          <button class="dir-btn" @click="selectOutput">浏览</button>
        </div>
      </div>
    </div>
    <div class="convert-section">
      <button class="convert-btn" :disabled="!filePath || !startTime || !endTime || processing" @click="startClip">{{ processing ? '截取中...' : '开始截取' }}</button>
      <ProgressBar :progress="progress" :text="progressText" />
    </div>
    <LogPanel title="截取日志" :logs="logs" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { selectVideoFile, selectFolder, clipVideo, onEvent } from '../utils/api'
import type { ProgressPayload, LogPayload } from '../utils/api'
import type { LogEntry } from '../components/LogPanel.vue'
import Icon from '../components/Icon.vue'
import ProgressBar from '../components/ProgressBar.vue'
import LogPanel from '../components/LogPanel.vue'

const filePath = ref(''); const fileName = ref(''); const startTime = ref(''); const endTime = ref('')
const outputDir = ref(''); const processing = ref(false); const progress = ref(0); const progressText = ref('')
const logs = ref<LogEntry[]>([])

async function selectFile() { try { const f = await selectVideoFile(); if (f) { filePath.value = f; fileName.value = f.split('\\').pop() || f; logs.value.push({ time: new Date().toLocaleTimeString(), message: `已选择: ${fileName.value}`, type: 'info' }) } } catch (e: any) { logs.value.push({ time: new Date().toLocaleTimeString(), message: `选择失败: ${e.message}`, type: 'error' }) } }
async function selectOutput() { try { const d = await selectFolder(); if (d) { outputDir.value = d; logs.value.push({ time: new Date().toLocaleTimeString(), message: `输出目录: ${d}`, type: 'info' }) } } catch (e: any) { logs.value.push({ time: new Date().toLocaleTimeString(), message: `目录选择失败: ${e.message}`, type: 'error' }) } }
async function startClip() {
  if (!filePath.value || !startTime.value || !endTime.value) return
  processing.value = true; progress.value = 0; logs.value = []
  const up = await onEvent<ProgressPayload>('clip-video-progress', (d) => { if (d.percent) { progress.value = Math.min(99, d.percent); progressText.value = `${Math.round(d.percent)}%` } })
  const ul = await onEvent<LogPayload>('clip-video-log', (d) => { logs.value.push({ time: d.time, message: d.message, type: d.type }) })
  try {
    const r = await clipVideo(filePath.value, startTime.value, endTime.value, outputDir.value || undefined)
    if (r.success) { progress.value = 100; progressText.value = '完成' }
    else { logs.value.push({ time: new Date().toLocaleTimeString(), message: `失败: ${r.error}`, type: 'error' }) }
  } catch (e: any) { logs.value.push({ time: new Date().toLocaleTimeString(), message: e.message, type: 'error' }) }
  finally { processing.value = false; up(); ul() }
}
</script>
