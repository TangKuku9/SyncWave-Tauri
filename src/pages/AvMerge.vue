<template>
  <div class="page-container active av-merge-page">
    <div class="content-header">
      <h2>音视频合并</h2>
      <span class="badge">ffmpeg</span>
    </div>

    <div class="file-select-container">
      <div class="file-select-box" @click="selectVideo">
        <Icon name="videoFile" :size="40" />
        <div class="file-select-label">视频文件</div>
        <div class="file-select-hint">点击选择 MP4/MKV/AVI 文件</div>
        <div class="file-select-info" v-if="videoFileName">{{ videoFileName }}</div>
        <button class="file-select-btn">选择视频</button>
      </div>
      <div class="file-select-box" @click="selectAudio">
        <Icon name="musicNote" :size="40" />
        <div class="file-select-label">音频文件</div>
        <div class="file-select-hint">点击选择 M4A/AAC/MP3 文件</div>
        <div class="file-select-info" v-if="audioFileName">{{ audioFileName }}</div>
        <button class="file-select-btn">选择音频</button>
      </div>
    </div>

    <div class="dir-settings">
      <div class="dir-group">
        <div class="dir-label">输出目录</div>
        <div class="dir-input-row">
          <input type="text" class="dir-input" v-model="outputDir" placeholder="默认：视频文件所在目录" />
          <button class="dir-btn" @click="selectOutput">浏览</button>
        </div>
      </div>
    </div>

    <div class="convert-section">
      <button class="convert-btn" :disabled="!videoPath || !audioPath || processing" @click="startMerge">
        {{ processing ? '合并中...' : '开始合并' }}
      </button>
      <ProgressBar :progress="progress" :text="progressText" />
    </div>

    <LogPanel title="合并日志" :logs="logs" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { selectVideoFile, selectAudioFile, selectFolder, mergeAv, onEvent } from '../utils/api'
import type { FfmpegResult, ProgressPayload, LogPayload } from '../utils/api'
import type { LogEntry } from '../components/LogPanel.vue'
import Icon from '../components/Icon.vue'
import ProgressBar from '../components/ProgressBar.vue'
import LogPanel from '../components/LogPanel.vue'

const videoPath = ref('')
const videoFileName = ref('')
const audioPath = ref('')
const audioFileName = ref('')
const outputDir = ref('')
const processing = ref(false)
const progress = ref(0)
const progressText = ref('')
const logs = ref<LogEntry[]>([])

async function selectVideo() {
  const f = await selectVideoFile()
  if (f) { videoPath.value = f; videoFileName.value = f.split('\\').pop() || f }
}
async function selectAudio() {
  const f = await selectAudioFile()
  if (f) { audioPath.value = f; audioFileName.value = f.split('\\').pop() || f }
}
async function selectOutput() {
  const d = await selectFolder()
  if (d) outputDir.value = d
}

async function startMerge() {
  if (!videoPath.value || !audioPath.value) return
  processing.value = true; progress.value = 0; logs.value = []
  const unlistenP = await onEvent<ProgressPayload>('merge-progress', (d) => {
    if (d.percent) { progress.value = Math.min(99, d.percent); progressText.value = `${Math.round(d.percent)}%` }
  })
  const unlistenL = await onEvent<LogPayload>('merge-log', (d) => { logs.value.push({ time: d.time, message: d.message, type: d.type }) })
  try {
    const r = await mergeAv(videoPath.value, audioPath.value, outputDir.value || undefined)
    if (r.success) { progress.value = 100; progressText.value = '完成' }
    else { logs.value.push({ time: new Date().toLocaleTimeString(), message: `失败: ${r.error}`, type: 'error' }) }
  } catch (e: any) { logs.value.push({ time: new Date().toLocaleTimeString(), message: e.message, type: 'error' }) }
  finally { processing.value = false; unlistenP(); unlistenL() }
}
</script>
