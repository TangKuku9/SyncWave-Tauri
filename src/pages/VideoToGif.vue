<template>
  <div class="page-container active video-to-gif-page">
    <div class="content-header"><h2>视频转GIF</h2><span class="badge">ffmpeg</span></div>
    <div class="file-select-container">
      <div class="file-select-box" @click="selectFile">
        <Icon name="videoCamera" :size="40" />
        <div class="file-select-label">选择视频文件</div>
        <div class="file-select-hint">点击选择要转换的视频</div>
        <div class="file-select-info" v-if="fileName">{{ fileName }}</div>
        <button class="file-select-btn">选择视频</button>
      </div>
    </div>
    <div class="dir-settings">
      <div class="dir-group">
        <div class="dir-label">帧率 (FPS)</div>
        <div class="dir-input-row">
          <input type="range" class="range-slider" min="1" max="30" v-model.number="fps" />
          <span class="range-value">{{ fps }}</span>
        </div>
      </div>
      <div class="dir-group">
        <div class="dir-label">GIF宽度</div>
        <select class="dir-input" v-model.number="width">
          <option :value="240">240px (小)</option><option :value="320">320px (中)</option>
          <option :value="480">480px (大)</option><option :value="640">640px (高清)</option>
          <option :value="800">800px (超清)</option>
        </select>
      </div>
      <div class="dir-group">
        <div class="dir-label">画幅比例</div>
        <select class="dir-input" v-model="aspectRatio">
          <option value="original">原始比例</option><option value="1:1">方形 1:1</option>
          <option value="4:3">标准 4:3</option><option value="16:9">宽屏 16:9</option>
          <option value="9:16">竖屏 9:16</option>
        </select>
      </div>
      <div class="dir-group">
        <div class="dir-label">开始时间</div>
        <input type="text" class="dir-input" v-model="startTime" placeholder="00:00:00 (可选)" />
      </div>
      <div class="dir-group">
        <div class="dir-label">结束时间</div>
        <input type="text" class="dir-input" v-model="endTime" placeholder="00:00:00 (可选)" />
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
      <button class="convert-btn" :disabled="!filePath || processing" @click="startConvert">{{ processing ? '转换中...' : '开始转换' }}</button>
      <ProgressBar :progress="progress" :text="progressText" />
    </div>
    <LogPanel title="转换日志" :logs="logs" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { selectVideoFile, selectFolder, videoToGif, onEvent } from '../utils/api'
import type { ProgressPayload, LogPayload } from '../utils/api'
import type { LogEntry } from '../components/LogPanel.vue'
import Icon from '../components/Icon.vue'
import ProgressBar from '../components/ProgressBar.vue'
import LogPanel from '../components/LogPanel.vue'

const filePath = ref(''); const fileName = ref(''); const fps = ref(10); const width = ref(320)
const aspectRatio = ref('original'); const startTime = ref(''); const endTime = ref('')
const outputDir = ref(''); const processing = ref(false); const progress = ref(0); const progressText = ref('')
const logs = ref<LogEntry[]>([])

async function selectFile() { const f = await selectVideoFile(); if (f) { filePath.value = f; fileName.value = f.split('\\').pop() || f } }
async function selectOutput() { const d = await selectFolder(); if (d) outputDir.value = d }
async function startConvert() {
  if (!filePath.value) return
  processing.value = true; progress.value = 0; logs.value = []
  const up = await onEvent<ProgressPayload>('video-to-gif-progress', (d) => { if (d.percent) { progress.value = Math.min(99, d.percent); progressText.value = `${Math.round(d.percent)}%` } })
  const ul = await onEvent<LogPayload>('video-to-gif-log', (d) => { logs.value.push({ time: d.time, message: d.message, type: d.type }) })
  try {
    const r = await videoToGif(filePath.value, fps.value, width.value, aspectRatio.value, startTime.value || undefined, endTime.value || undefined, outputDir.value || undefined)
    if (r.success) { progress.value = 100; progressText.value = '完成' }
    else { logs.value.push({ time: new Date().toLocaleTimeString(), message: `失败: ${r.error}`, type: 'error' }) }
  } catch (e: any) { logs.value.push({ time: new Date().toLocaleTimeString(), message: e.message, type: 'error' }) }
  finally { processing.value = false; up(); ul() }
}
</script>
