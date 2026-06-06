<template>
  <div class="page-container active extract-audio-page">
    <div class="content-header"><h2>音频提取</h2><span class="badge">ffmpeg</span></div>
    <div class="file-select-container">
      <div class="file-select-box" @click="selectFile">
        <Icon name="videoCamera" :size="40" />
        <div class="file-select-label">选择视频文件</div>
        <div class="file-select-hint">点击选择要提取音频的视频</div>
        <div class="file-select-info" v-if="fileName">{{ fileName }}</div>
        <button class="file-select-btn">选择视频</button>
      </div>
    </div>
    <div class="dir-settings">
      <div class="dir-group">
        <div class="dir-label">输出格式</div>
        <select class="dir-input" v-model="audioFormat">
          <option value="mp3">MP3</option><option value="flac">FLAC</option>
          <option value="wav">WAV</option><option value="m4a">M4A</option><option value="ogg">OGG</option>
        </select>
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
      <button class="convert-btn" :disabled="!filePath || processing" @click="startExtract">{{ processing ? '提取中...' : '提取音频' }}</button>
      <ProgressBar :progress="progress" :text="progressText" />
    </div>
    <LogPanel title="提取日志" :logs="logs" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { selectVideoFile, selectFolder, extractAudio, onEvent } from '../utils/api'
import type { ProgressPayload, LogPayload } from '../utils/api'
import type { LogEntry } from '../components/LogPanel.vue'
import Icon from '../components/Icon.vue'
import ProgressBar from '../components/ProgressBar.vue'
import LogPanel from '../components/LogPanel.vue'

const filePath = ref(''); const fileName = ref(''); const audioFormat = ref('mp3')
const outputDir = ref(''); const processing = ref(false); const progress = ref(0); const progressText = ref('')
const logs = ref<LogEntry[]>([])

async function selectFile() { const f = await selectVideoFile(); if (f) { filePath.value = f; fileName.value = f.split('\\').pop() || f } }
async function selectOutput() { const d = await selectFolder(); if (d) outputDir.value = d }
async function startExtract() {
  if (!filePath.value) return
  processing.value = true; progress.value = 0; logs.value = []
  const up = await onEvent<ProgressPayload>('extract-audio-progress', (d) => { if (d.percent) { progress.value = Math.min(99, d.percent); progressText.value = `${Math.round(d.percent)}%` } })
  const ul = await onEvent<LogPayload>('extract-audio-log', (d) => { logs.value.push({ time: d.time, message: d.message, type: d.type }) })
  try {
    const r = await extractAudio(filePath.value, audioFormat.value, outputDir.value || undefined)
    if (r.success) { progress.value = 100; progressText.value = '完成' }
    else { logs.value.push({ time: new Date().toLocaleTimeString(), message: `失败: ${r.error}`, type: 'error' }) }
  } catch (e: any) { logs.value.push({ time: new Date().toLocaleTimeString(), message: e.message, type: 'error' }) }
  finally { processing.value = false; up(); ul() }
}
</script>
