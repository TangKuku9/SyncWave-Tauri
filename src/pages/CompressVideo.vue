<template>
  <div class="page-container active compress-video-page">
    <div class="content-header"><h2>视频压缩</h2><span class="badge">ffmpeg</span></div>
    <div class="file-select-container">
      <div class="file-select-box" :class="{ 'has-file': filePath }" @click="selectFile">
        <Icon name="compress" :size="40" />
        <div class="file-select-label">选择视频文件</div>
        <div class="file-select-hint">点击选择要压缩的视频</div>
        <div class="file-select-info" v-if="fileName">{{ fileName }}</div>
        <button class="file-select-btn">选择视频</button>
      </div>
    </div>
    <div class="dir-settings">
      <div class="dir-group">
        <div class="dir-label">压缩质量 (CRF)</div>
        <select class="dir-input" v-model="crf">
          <option value="18">极高 (CRF 18)</option><option value="23">高 (CRF 23)</option>
          <option value="28">中 (CRF 28)</option><option value="33">低 (CRF 33)</option>
        </select>
      </div>
      <div class="dir-group">
        <div class="dir-label">输出分辨率</div>
        <select class="dir-input" v-model="scale">
          <option value="original">保持原分辨率</option><option value="1920:-1">1920x1080 (1080p)</option>
          <option value="1280:-1">1280x720 (720p)</option><option value="854:-1">854x480 (480p)</option>
          <option value="640:-1">640x360 (360p)</option>
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
      <button class="convert-btn" :disabled="!filePath || processing" @click="startCompress">{{ processing ? '压缩中...' : '开始压缩' }}</button>
      <ProgressBar :progress="progress" :text="progressText" />
    </div>
    <LogPanel title="压缩日志" :logs="logs" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { selectVideoFile, selectFolder, compressVideo, onEvent } from '../utils/api'
import type { ProgressPayload, LogPayload } from '../utils/api'
import type { LogEntry } from '../components/LogPanel.vue'
import Icon from '../components/Icon.vue'
import ProgressBar from '../components/ProgressBar.vue'
import LogPanel from '../components/LogPanel.vue'

const filePath = ref(''); const fileName = ref(''); const crf = ref('23'); const scale = ref('original')
const outputDir = ref(''); const processing = ref(false); const progress = ref(0); const progressText = ref('')
const logs = ref<LogEntry[]>([])

async function selectFile() { try { const f = await selectVideoFile(); if (f) { filePath.value = f; fileName.value = f.split('\\').pop() || f; logs.value.push({ time: new Date().toLocaleTimeString(), message: `已选择: ${fileName.value}`, type: 'info' }) } } catch (e: any) { logs.value.push({ time: new Date().toLocaleTimeString(), message: `选择失败: ${e.message}`, type: 'error' }) } }
async function selectOutput() { try { const d = await selectFolder(); if (d) { outputDir.value = d; logs.value.push({ time: new Date().toLocaleTimeString(), message: `输出目录: ${d}`, type: 'info' }) } } catch (e: any) { logs.value.push({ time: new Date().toLocaleTimeString(), message: `目录选择失败: ${e.message}`, type: 'error' }) } }
async function startCompress() {
  if (!filePath.value) return
  processing.value = true; progress.value = 0; logs.value = []
  const up = await onEvent<ProgressPayload>('compress-video-progress', (d) => { if (d.percent) { progress.value = Math.min(99, d.percent); progressText.value = `${Math.round(d.percent)}%` } })
  const ul = await onEvent<LogPayload>('compress-video-log', (d) => { logs.value.push({ time: d.time, message: d.message, type: d.type }) })
  try {
    const r = await compressVideo(filePath.value, crf.value, scale.value === 'original' ? undefined : scale.value, outputDir.value || undefined)
    if (r.success) { progress.value = 100; progressText.value = '完成' }
    else { logs.value.push({ time: new Date().toLocaleTimeString(), message: `失败: ${r.error}`, type: 'error' }) }
  } catch (e: any) { logs.value.push({ time: new Date().toLocaleTimeString(), message: e.message, type: 'error' }) }
  finally { processing.value = false; up(); ul() }
}
</script>
