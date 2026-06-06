<template>
  <div class="page-container active ncm-page">
    <div class="content-header">
      <h2>NCM 格式转换</h2>
      <span class="badge">unlock-music</span>
    </div>

    <!-- 拖拽区域 -->
    <DropZone
      icon-name="folder"
      text="拖拽 NCM 文件到此处"
      hint="或点击下方按钮选择文件"
      button-text="选择文件"
      @select="selectFiles"
    />

    <!-- 目录设置 -->
    <div class="dir-settings">
      <div class="dir-group">
        <div class="dir-label">输出目录</div>
        <div class="dir-input-row">
          <input type="text" class="dir-input" v-model="outputDir" placeholder="默认：原文件所在目录" />
          <button class="dir-btn" @click="selectOutputDir">浏览</button>
        </div>
      </div>
    </div>

    <!-- 文件列表 -->
    <div class="file-list-section">
      <div class="section-header">
        <span class="section-title">待转换文件</span>
        <span class="file-count">{{ files.length }} 个文件</span>
      </div>
      <div class="file-list">
        <div class="file-list-empty" v-if="files.length === 0">暂无文件，请拖拽或选择NCM文件</div>
        <div v-for="(file, i) in files" :key="i" class="file-list-item">
          <span class="file-name">{{ file.name }}</span>
          <span :class="['file-status', file.status]">
            {{ file.status === 'pending' ? '待转换' : file.status === 'converting' ? '转换中...' : file.status === 'success' ? '✓ 成功' : '✗ 失败' }}
          </span>
        </div>
      </div>
    </div>

    <!-- 转换选项和进度 -->
    <div class="convert-section">
      <div class="convert-options">
        <button class="convert-option-btn" :disabled="files.length === 0 || converting" @click="startConvert('mp3')">
          <Icon name="musicNote" :size="16" /> <span>转 MP3</span>
        </button>
        <button class="convert-option-btn" :disabled="files.length === 0 || converting" @click="startConvert('flac')">
          <Icon name="musicNote" :size="16" /> <span>转 FLAC</span>
        </button>
        <button class="convert-option-btn" :disabled="files.length === 0 || converting" @click="startConvert('lyrics')">
          <Icon name="fileText" :size="16" /> <span>提取歌词</span>
        </button>
      </div>
      <ProgressBar :progress="progress" :text="progressText" />
    </div>

    <!-- 日志区域 -->
    <LogPanel title="转换日志" :logs="logs" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { selectFiles as apiSelectFiles, selectFolder, convertNcm, onEvent } from '../utils/api'
import type { NcmConvertProgress, NcmConvertLog, NcmConvertResult } from '../utils/api'
import type { LogEntry } from '../components/LogPanel.vue'
import DropZone from '../components/DropZone.vue'
import ProgressBar from '../components/ProgressBar.vue'
import LogPanel from '../components/LogPanel.vue'
import Icon from '../components/Icon.vue'

interface FileItem {
  name: string
  path: string
  status: 'pending' | 'converting' | 'success' | 'error'
}

const files = ref<FileItem[]>([])
const outputDir = ref('')
const converting = ref(false)
const progress = ref(0)
const progressText = ref('')
const logs = ref<LogEntry[]>([])

let unlisteners: (() => void)[] = []

onMounted(async () => {
  unlisteners.push(await onEvent<NcmConvertProgress>('convert-progress', (data) => {
    if (data.status === 'converting') {
      const idx = data.index
      if (idx < files.value.length) files.value[idx].status = 'converting'
      progress.value = Math.round((data.index / data.total) * 100)
      progressText.value = `${data.index + 1} / ${data.total}`
    } else if (data.status === 'success' || data.status === 'error') {
      const idx = data.index
      if (idx < files.value.length) files.value[idx].status = data.status as any
    }
  }))

  unlisteners.push(await onEvent<NcmConvertLog>('convert-log', (data) => {
    logs.value.push({ time: data.time, message: data.message, type: data.type })
  }))

  unlisteners.push(await onEvent<NcmConvertResult[]>('convert-complete', () => {
    converting.value = false
    progress.value = 100
    progressText.value = '全部完成'
  }))
})

onUnmounted(() => {
  unlisteners.forEach(fn => fn())
})

async function selectFiles() {
  const selected = await apiSelectFiles()
  if (selected && selected.length > 0) {
    const existing = new Set(files.value.map(f => f.path))
    for (const p of selected) {
      if (!existing.has(p)) {
        const name = p.split('\\').pop()?.split('/').pop() || p
        files.value.push({ name, path: p, status: 'pending' })
      }
    }
    logs.value.push({ time: new Date().toLocaleTimeString(), message: `添加了 ${selected.length} 个文件`, type: 'info' })
  }
}

async function selectOutputDir() {
  const dir = await selectFolder()
  if (dir) outputDir.value = dir
}

async function startConvert(type: string) {
  if (files.value.length === 0 || converting.value) return

  converting.value = true
  progress.value = 0
  progressText.value = '准备中...'
  logs.value = []
  files.value.forEach(f => f.status = 'pending')

  try {
    const filePaths = files.value.map(f => f.path)
    await convertNcm(filePaths, outputDir.value || undefined, type)
  } catch (e: any) {
    logs.value.push({ time: new Date().toLocaleTimeString(), message: `错误: ${e.message || e}`, type: 'error' })
    converting.value = false
  }
}
</script>

<style scoped>
.content-header { display: flex; align-items: center; gap: 12px; flex-shrink: 0; }
.content-header h2 { font-size: 22px; font-weight: 600; }

.dir-settings { display: flex; gap: 20px; flex-wrap: wrap; flex-shrink: 0; }
.dir-group { flex: 1; min-width: 280px; }
.dir-label { font-size: 13px; color: var(--text-secondary); margin-bottom: 8px; font-weight: 500; }
.dir-input-row { display: flex; gap: 8px; }
.dir-input {
  flex: 1; padding: 10px 14px; background: var(--bg-tertiary); border: 1px solid var(--border);
  border-radius: var(--border-radius-md); color: var(--text-primary); font-size: 13px; outline: none;
}
.dir-input:focus { border-color: var(--accent); }
.dir-input::placeholder { color: var(--text-muted); }
.dir-btn {
  padding: 10px 16px; background: var(--bg-tertiary); border: 1px solid var(--border);
  border-radius: var(--border-radius-md); color: var(--text-secondary); cursor: pointer; font-size: 13px;
}
.dir-btn:hover { border-color: var(--accent); color: var(--text-primary); }

.file-list-section { display: flex; flex-direction: column; flex-shrink: 0; min-height: 100px; max-height: 200px; }
.section-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
.section-title { font-size: 14px; font-weight: 600; color: var(--text-secondary); }
.file-count { font-size: 12px; color: var(--text-muted); }
.file-list {
  flex: 1; overflow-y: auto; background: var(--bg-card); border-radius: var(--border-radius-lg);
  border: 1px solid var(--border); min-height: 80px; padding: 4px 0;
}
.file-list-empty {
  display: flex; align-items: center; justify-content: center; height: 100%; min-height: 80px;
  color: var(--text-muted); font-size: 13px;
}
.file-list-item {
  display: flex; align-items: center; justify-content: space-between; padding: 8px 16px;
  border-left: 3px solid transparent;
}
.file-name { font-size: 13px; color: var(--text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1; }
.file-status { font-size: 12px; margin-left: 12px; flex-shrink: 0; }
.file-status.pending { color: var(--text-muted); }
.file-status.converting { color: var(--warning); }
.file-status.success { color: var(--success); }
.file-status.error { color: var(--error); }

.convert-section { display: flex; flex-direction: column; gap: 12px; flex-shrink: 0; }
.convert-options { display: flex; gap: 12px; flex-wrap: wrap; }
.convert-option-btn {
  padding: 12px 24px; background: linear-gradient(135deg, var(--accent), #8b5cf6);
  color: white; border: none; border-radius: var(--border-radius-md); font-size: 14px;
  cursor: pointer; font-weight: 500; display: flex; align-items: center; gap: 8px;
  transition: all 0.2s ease;
}
.convert-option-btn:hover:not(:disabled) { transform: translateY(-2px); box-shadow: 0 8px 25px var(--accent-glow); }
.convert-option-btn:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
