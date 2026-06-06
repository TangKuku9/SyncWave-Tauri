<template>
  <div class="app-root">
    <!-- 自定义标题栏 -->
    <div class="titlebar">
      <div class="titlebar-left" data-tauri-drag-region>
        <span class="titlebar-icon"><Icon name="musicNote" :size="16" /></span>
        <span class="titlebar-title">SyncWave</span>
      </div>
      <div class="titlebar-controls">
        <button class="titlebar-btn" @click="minimizeWindow"><Icon name="minimize" /></button>
        <button class="titlebar-btn" @click="toggleMaximize">
          <Icon :name="isMaximized ? 'restore' : 'maximize'" />
        </button>
        <button class="titlebar-btn close-btn" @click="closeWindow"><Icon name="close" /></button>
      </div>
    </div>

    <div class="app-container">
      <!-- 侧边栏 -->
      <div class="sidebar">
        <div class="sidebar-header">工具列表</div>
        <div
          v-for="item in mainTools" :key="item.id"
          class="sidebar-item" :class="{ active: activePage === item.id }"
          @click="activePage = item.id"
        >
          <span class="sidebar-item-icon"><Icon :name="item.icon" /></span>
          <span class="sidebar-item-text">{{ item.label }}</span>
        </div>

        <div class="sidebar-divider"></div>
        <div class="sidebar-header">ffmpeg 工具</div>
        <div
          v-for="item in ffmpegTools" :key="item.id"
          class="sidebar-item" :class="{ active: activePage === item.id }"
          @click="activePage = item.id"
        >
          <span class="sidebar-item-icon"><Icon :name="item.icon" /></span>
          <span class="sidebar-item-text">{{ item.label }}</span>
        </div>

        <div class="sidebar-divider"></div>
        <div class="sidebar-item sidebar-item-add" :class="{ active: activePage === 'mesugaki' }" @click="activePage = 'mesugaki'">
          <span class="sidebar-item-icon"><Icon name="plus" /></span>
          <span class="sidebar-item-text">添加工具</span>
        </div>
      </div>

      <!-- 主内容区 -->
      <div class="main-content">
        <NcmConvert v-show="activePage === 'ncm-convert'" />
        <MusicPlayer v-show="activePage === 'music-player'" />
        <AvMerge v-show="activePage === 'av-merge'" />
        <FormatConvert v-show="activePage === 'format-convert'" />
        <ExtractAudio v-show="activePage === 'extract-audio'" />
        <ClipVideo v-show="activePage === 'clip-video'" />
        <CompressVideo v-show="activePage === 'compress-video'" />
        <VideoToGif v-show="activePage === 'video-to-gif'" />
        <Mesugaki v-show="activePage === 'mesugaki'" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import Icon from './components/Icon.vue'
import NcmConvert from './pages/NcmConvert.vue'
import MusicPlayer from './pages/MusicPlayer.vue'
import AvMerge from './pages/AvMerge.vue'
import FormatConvert from './pages/FormatConvert.vue'
import ExtractAudio from './pages/ExtractAudio.vue'
import ClipVideo from './pages/ClipVideo.vue'
import CompressVideo from './pages/CompressVideo.vue'
import VideoToGif from './pages/VideoToGif.vue'
import Mesugaki from './pages/Mesugaki.vue'

const activePage = ref('ncm-convert')
const isMaximized = ref(false)

const mainTools = [
  { id: 'ncm-convert', icon: 'musicNote', label: 'NCM 转换' },
  { id: 'music-player', icon: 'headphones', label: '音乐播放器' },
]
const ffmpegTools = [
  { id: 'av-merge', icon: 'videoCamera', label: '音视频合并' },
  { id: 'format-convert', icon: 'arrowsRotate', label: '格式转换' },
  { id: 'extract-audio', icon: 'speaker', label: '音频提取' },
  { id: 'clip-video', icon: 'scissors', label: '视频截取' },
  { id: 'compress-video', icon: 'compress', label: '视频压缩' },
  { id: 'video-to-gif', icon: 'image', label: '视频转GIF' },
]

const appWindow = getCurrentWindow()
async function minimizeWindow() { await appWindow.minimize() }
async function toggleMaximize() { await appWindow.toggleMaximize(); isMaximized.value = await appWindow.isMaximized() }
async function closeWindow() { await appWindow.close() }
</script>

<style scoped>
.app-root { display: flex; flex-direction: column; height: 100vh; }
</style>
