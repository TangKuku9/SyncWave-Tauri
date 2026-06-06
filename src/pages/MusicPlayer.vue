<template>
  <div class="page-container active music-player-page">
    <div class="content-header">
      <h2>音乐播放器</h2>
      <span class="badge">本地音乐</span>
    </div>

    <!-- 顶部控制栏 -->
    <div class="music-controls-top">
      <button class="add-music-btn" @click="addMusic">
        <Icon name="plus" :size="16" /> <span>添加音乐</span>
      </button>
      <button class="add-music-btn scan-btn" @click="scanDir">
        <Icon name="folder" :size="16" /> <span>扫描目录</span>
      </button>
      <div class="search-box">
        <input type="text" v-model="searchQuery" placeholder="在歌单中搜索..." />
      </div>
    </div>

    <!-- 播放列表/歌词视图 -->
    <div class="playlist-views">
      <!-- 播放列表 -->
      <div class="playlist-view" :class="{ active: currentView === 'playlist' }">
        <!-- 歌单标签栏 -->
        <div class="playlist-tabs-bar">
          <div class="tabs-scroll-container" ref="tabsScrollRef">
            <div class="tabs-list"
              @dragstart="onDragStart" @dragover="onDragOver"
              @dragenter.prevent @dragleave="onDragLeave"
              @drop="onDrop" @dragend="onDragEnd">
              <div
                v-for="pl in playlistsIndex" :key="pl.id"
                class="tab-item"
                :class="{ active: pl.id === currentPlaylistId, dragging: dragId === pl.id,
                  'drop-target-left': dropTarget?.id === pl.id && dropTarget?.side === 'left',
                  'drop-target-right': dropTarget?.id === pl.id && dropTarget?.side === 'right' }"
                :data-playlist-id="pl.id"
                draggable="true"
                @click="switchPlaylist(pl.id)"
              >
                <span class="tab-name">{{ pl.name }}</span>
                <span v-if="!pl.is_default" class="tab-close" @click.stop="confirmDeletePlaylist(pl.id)">&times;</span>
              </div>
            </div>
          </div>
          <div class="tabs-actions">
            <button class="tabs-action-btn" @click="showCreateModal" title="新建歌单"><Icon name="plus" :size="16" /></button>
            <button class="tabs-action-btn" @click.stop="toggleMenu" title="更多操作"><Icon name="list" :size="16" /></button>
          </div>
        </div>

        <!-- 浮动菜单 -->
        <div class="playlist-context-menu" ref="contextMenuRef" v-show="menuOpen">
          <button class="context-menu-item" @click="handleMenuAction('import')">
            <span class="menu-icon"><Icon name="folder" :size="16" /></span>
            <span class="menu-text">导入歌单</span>
          </button>
          <button class="context-menu-item" @click="handleMenuAction('export-json')">
            <span class="menu-icon"><Icon name="musicNote" :size="16" /></span>
            <span class="menu-text">导出为 JSON</span>
          </button>
          <button class="context-menu-item" @click="handleMenuAction('export-m3u')">
            <span class="menu-icon"><Icon name="musicNote" :size="16" /></span>
            <span class="menu-text">导出为 M3U</span>
          </button>
          <div class="context-menu-divider"></div>
          <button class="context-menu-item" @click="handleMenuAction('rename')">
            <span class="menu-icon"><Icon name="musicNote" :size="16" /></span>
            <span class="menu-text">重命名歌单</span>
          </button>
          <button class="context-menu-item danger" @click="handleMenuAction('delete')">
            <span class="menu-icon"><Icon name="close" :size="16" /></span>
            <span class="menu-text">删除歌单</span>
          </button>
        </div>

        <!-- 模态弹窗 -->
        <div class="playlist-modal-overlay" v-show="modalOpen" @click.self="hideModal">
          <div class="playlist-modal">
            <div class="modal-header">
              <span class="modal-title">{{ modalTitle }}</span>
              <button class="modal-close-btn" @click="hideModal"><Icon name="close" :size="14" /></button>
            </div>
            <div class="modal-body">
              <input
                v-show="modalMode !== 'deleteConfirm'"
                type="text" class="modal-input" v-model="modalInput"
                :placeholder="modalMode === 'create' ? '输入歌单名称' : '输入新名称'"
                maxlength="50" @keydown.enter="handleModalConfirm"
                ref="modalInputRef"
              />
              <div class="modal-message danger" v-show="modalMode === 'deleteConfirm'" v-text="modalMessage"></div>
            </div>
            <div class="modal-footer">
              <button class="modal-btn cancel" @click="hideModal">取消</button>
              <button class="modal-btn" :class="modalMode === 'deleteConfirm' ? 'danger-btn' : 'confirm'" @click="handleModalConfirm">
                {{ modalMode === 'deleteConfirm' ? '删除' : '确定' }}
              </button>
            </div>
          </div>
        </div>

        <div class="playlist-header">
          <span class="playlist-title">播放列表</span>
          <span class="playlist-count">{{ playlist.length }} 首歌曲</span>
        </div>

        <div class="playlist-list">
          <div class="playlist-empty" v-if="playlist.length === 0">
            <div class="empty-icon"><Icon name="headphones" :size="48" /></div>
            <div class="empty-text">暂无音乐，请添加本地音频文件</div>
          </div>
          <div
            v-for="(song, i) in filteredPlaylist" :key="song.id"
            class="playlist-item" :class="{ playing: playlist.indexOf(song) === currentIndex && isPlaying }"
            @click="playSong(i)"
          >
            <span class="playlist-item-index">{{ playlist.indexOf(song) + 1 }}</span>
            <span class="playlist-item-icon"><Icon :name="playlist.indexOf(song) === currentIndex && isPlaying ? 'headphones' : 'musicNote'" :size="16" /></span>
            <div class="playlist-item-info">
              <div class="playlist-item-name" :title="song.name">{{ song.name }}</div>
              <div class="playlist-item-path" :title="song.path">{{ song.path }}</div>
            </div>
            <span class="playlist-item-duration">{{ song.duration ? formatTime(song.duration) : '--:--' }}</span>
            <button class="playlist-item-remove" @click.stop="removeSong(i)" title="移除"><Icon name="close" :size="14" /></button>
          </div>
        </div>
      </div>

      <!-- 歌词视图 -->
      <div class="lyrics-view" :class="{ active: currentView === 'lyrics' }">
        <div class="lyrics-header">
          <span class="lyrics-title">歌词</span>
        </div>
        <div class="lyrics-content" ref="lyricsContentRef">
          <div class="lyrics-empty" v-if="!currentLyrics || parsedLyrics.length === 0">暂无歌词</div>
          <div
            v-for="(line, i) in parsedLyrics" :key="i"
            class="lyrics-line" :class="{ active: currentLyricIndex === i }"
            @click="seekToLyric(line.time)"
          >{{ line.text }}</div>
        </div>
      </div>
    </div>

    <!-- 播放器控制区 -->
    <div class="player-controls">
      <div class="now-playing">
        <div class="album-art"><Icon name="musicNote" :size="28" /></div>
        <div class="song-info">
          <div class="song-title" @click="currentView = 'lyrics'">{{ currentSong?.name || '未播放' }}</div>
          <div class="song-artist">{{ currentSong ? (currentSong.path.split('\\').pop()?.split('/').pop() || '') : '未知艺术家' }}</div>
        </div>
      </div>

      <div class="player-center">
        <div class="control-buttons">
          <button class="control-btn" @click="prevSong"><Icon name="prev" :size="20" /></button>
          <button class="control-btn play-btn" @click="togglePlay">
            <Icon :name="isPlaying ? 'pause' : 'play'" :size="24" />
          </button>
          <button class="control-btn" @click="nextSong"><Icon name="next" :size="20" /></button>
        </div>
        <div class="progress-container">
          <span class="time">{{ formatTime(currentTime) }}</span>
          <div class="progress-slider" @click="seekProgress">
            <div class="progress-track">
              <div class="progress-fill" :style="{ width: (duration > 0 ? (currentTime / duration) * 100 : 0) + '%' }"></div>
            </div>
          </div>
          <span class="time">{{ formatTime(duration) }}</span>
        </div>
      </div>

      <div class="player-right">
        <button class="control-btn" @click="toggleMute" title="音量">
          <Icon :name="volumeIcon" :size="20" />
        </button>
        <div class="volume-slider">
          <input type="range" min="0" max="100" :value="Math.round(volume * 100)" @input="setVolume">
        </div>
        <button class="control-btn" :class="{ active: repeatMode !== 'none' }" @click="cycleRepeat" :title="repeatLabel">
          <Icon :name="repeatIcon" :size="20" />
        </button>
        <button class="control-btn" @click="toggleView(currentView === 'playlist' ? 'lyrics' : 'playlist')" title="播放列表/歌词">
          <Icon :name="currentView === 'playlist' ? 'lyrics' : 'list'" :size="20" />
        </button>
      </div>
    </div>

    <!-- Toast 通知 -->
    <Toast :message="toastMessage" :type="toastType" />

    <!-- 隐藏的 audio 元素 -->
    <audio ref="audioRef" @timeupdate="onTimeUpdate" @loadedmetadata="onMetadata" @ended="onEnded" @error="onError"></audio>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  selectMusicFiles, getMusicUrl, getAudioMetadata,
  readAudioFile,
  loadPlaylistsIndex, loadPlaylist, savePlaylist,
  createPlaylist as apiCreatePlaylist, deletePlaylist as apiDeletePlaylist,
  renamePlaylist as apiRenamePlaylist, reorderPlaylists as apiReorderPlaylists,
  selectFolder, scanDirectory, playWithFfplay,
  importPlaylist as apiImportPlaylist,
  exportPlaylistJson as apiExportJson, exportPlaylistM3u as apiExportM3u,
} from '../utils/api'
import type { Song, PlaylistIndexEntry } from '../utils/api'
import { formatTime, extractSongName } from '../utils/format'
import Icon from '../components/Icon.vue'
import Toast from '../components/Toast.vue'

// ============ 音频播放状态 ============
const audioRef = ref<HTMLAudioElement>()
const playlist = ref<Song[]>([])
const currentIndex = ref(-1)
const isPlaying = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const volume = ref(0.8)
const muted = ref(false)
const repeatMode = ref<'none' | 'all' | 'one' | 'shuffle'>('none')
const shuffleOrder = ref<number[]>([])
const shufflePos = ref(-1)
const searchQuery = ref('')
const currentView = ref<'playlist' | 'lyrics'>('playlist')
const currentLyrics = ref<string | null>(null)
const useExternalPlayer = ref(false)

// ============ Blob URL 缓存 ============
let currentBlobUrl: string | null = null
let _retryingBlob = false

function revokeBlobUrl(url: string | null) {
  if (url && url.startsWith('blob:')) {
    URL.revokeObjectURL(url)
  }
}

// ============ 歌单管理状态 ============
const playlistsIndex = ref<PlaylistIndexEntry[]>([])
const currentPlaylistId = ref('default')
const dirty = ref(false)
let autosaveTimer: number | undefined

// ============ 浮动菜单 ============
const menuOpen = ref(false)
const contextMenuRef = ref<HTMLElement>()

// ============ 模态弹窗 ============
const modalOpen = ref(false)
const modalMode = ref<'create' | 'rename' | 'deleteConfirm' | null>(null)
const modalTitle = ref('')
const modalInput = ref('')
const modalMessage = ref('')
const pendingDeleteId = ref<string | null>(null)
const modalInputRef = ref<HTMLInputElement>()

// ============ 拖拽排序 ============
const dragId = ref<string | null>(null)
const dropTarget = ref<{ id: string; side: 'left' | 'right' } | null>(null)
const tabsScrollRef = ref<HTMLElement>()

// ============ Toast ============
const toastMessage = ref('')
const toastType = ref<'success' | 'error' | 'warning' | 'info'>('info')
function showToast(msg: string, type: 'success' | 'error' | 'warning' | 'info' = 'info') {
  toastMessage.value = msg
  toastType.value = type
}

// ============ 外部播放器 ============
const lyricsContentRef = ref<HTMLElement>()

function toggleExternalPlayer() {
  useExternalPlayer.value = !useExternalPlayer.value
  if (useExternalPlayer.value) showToast('已切换到外部播放(ffplay)', 'info')
  else showToast('已切换到内置播放', 'info')
}

const repeatIcons: Record<string, string> = { none: 'repeatOff', all: 'repeat', one: 'repeatOne', shuffle: 'shuffle' }
const repeatLabels: Record<string, string> = { none: '顺序播放', all: '列表循环', one: '单曲循环', shuffle: '随机播放' }

const currentSong = computed(() => currentIndex.value >= 0 ? playlist.value[currentIndex.value] : null)
const repeatIcon = computed(() => repeatIcons[repeatMode.value])
const repeatLabel = computed(() => repeatLabels[repeatMode.value])
const volumeIcon = computed(() => muted.value || volume.value === 0 ? 'volumeMute' : volume.value < 0.5 ? 'volumeLow' : 'volumeHigh')

const filteredPlaylist = computed(() => {
  if (!searchQuery.value) return playlist.value
  const q = searchQuery.value.toLowerCase()
  return playlist.value.filter(s => s.name.toLowerCase().includes(q))
})

interface LyricLine { time: number; text: string }
const parsedLyrics = computed<LyricLine[]>(() => {
  if (!currentLyrics.value) return []
  const lines: LyricLine[] = []
  const regex = /\[(\d+):(\d+\.?\d*)\](.*)/g
  let match
  while ((match = regex.exec(currentLyrics.value)) !== null) {
    const time = parseInt(match[1]) * 60 + parseFloat(match[2])
    lines.push({ time, text: match[3].trim() })
  }
  return lines.sort((a, b) => a.time - b.time)
})

const currentLyricIndex = computed(() => {
  let idx = -1
  for (let i = 0; i < parsedLyrics.value.length; i++) {
    if (parsedLyrics.value[i].time <= currentTime.value) idx = i
    else break
  }
  return idx
})

// ============ 歌单加载 (Electron 的 init) ============

onMounted(async () => {
  try {
    playlistsIndex.value = await loadPlaylistsIndex()
    const defaultPl = playlistsIndex.value.find(p => p.is_default) || playlistsIndex.value[0]
    if (defaultPl) await switchPlaylist(defaultPl.id)
  } catch (e) { /* ignore */ }
})

// 点击外部关闭菜单
function onDocClick(e: MouseEvent) {
  if (menuOpen.value && contextMenuRef.value && !contextMenuRef.value.contains(e.target as Node)) {
    menuOpen.value = false
  }
}
onMounted(() => document.addEventListener('click', onDocClick))
onUnmounted(() => document.removeEventListener('click', onDocClick))

// ============ 歌单切换 (Electron switchPlaylist) ============

async function switchPlaylist(targetId: string) {
  try {
    // 保存当前歌单
    if (dirty.value) await saveCurrentPlaylist()
    // 暂停播放并清理 blob URL
    if (audioRef.value) {
      audioRef.value.pause()
      audioRef.value.src = ''
      isPlaying.value = false
      currentIndex.value = -1
    }
    if (currentBlobUrl) { revokeBlobUrl(currentBlobUrl); currentBlobUrl = null }
    const pl = await loadPlaylist(targetId)
    currentPlaylistId.value = targetId
    playlist.value = (pl.songs || []).map(s => ({
      id: s.id, path: s.path, name: s.name, duration: s.duration || 0
    }))
    currentIndex.value = -1
    dirty.value = false
  } catch (e) { /* ignore */ }
}

// ============ 添加音乐 (Electron addMusicFiles) ============

async function addMusic() {
  try {
    const files = await selectMusicFiles()
    if (!files || files.length === 0) return
    const audioExts = ['.mp3', '.flac', '.wav', '.ogg', '.m4a', '.aac', '.wma']
    const existingPaths = new Set(playlist.value.map(s => s.path))
    let added = 0
    for (const f of files) {
      const ext = f.substring(f.lastIndexOf('.')).toLowerCase()
      if (!audioExts.includes(ext)) continue
      if (existingPaths.has(f)) continue
      const name = f.split('\\').pop()?.split('/').pop()?.replace(/\.[^/.]+$/, '') || f
      playlist.value.push({ id: `song-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, path: f, name, duration: 0 })
      existingPaths.add(f)
      added++
    }
    if (added > 0) {
      if (repeatMode.value === 'shuffle') generateShuffleOrder()
      showToast(`成功添加 ${added} 首歌曲`, 'success')
      markDirty()
    }
  } catch (e) { /* ignore */ }
}

async function scanDir() {
  try {
    const dir = await selectFolder()
    if (!dir) return
    const files = await scanDirectory(dir)
    if (files.length === 0) { showToast('未找到音频文件', 'warning'); return }
    const existingPaths = new Set(playlist.value.map(s => s.path))
    let added = 0
    for (const f of files) {
      if (existingPaths.has(f)) continue
      const name = f.split('\\').pop()?.split('/').pop()?.replace(/\.[^/.]+$/, '') || f
      playlist.value.push({ id: `song-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, path: f, name, duration: 0 })
      existingPaths.add(f)
      added++
    }
    if (added > 0) { showToast(`成功添加 ${added} 首歌曲`, 'success'); markDirty() }
    else showToast('未找到新的音频文件', 'warning')
  } catch (e) { /* ignore */ }
}

// ============ 删除歌曲 (Electron removeSong) ============

function removeSong(index: number) {
  const realIndex = playlist.value.indexOf(filteredPlaylist.value[index])
  if (realIndex < 0) return
  if (realIndex === currentIndex.value) {
    audioRef.value?.pause()
    if (audioRef.value) audioRef.value.src = ''
    if (currentBlobUrl) { revokeBlobUrl(currentBlobUrl); currentBlobUrl = null }
    isPlaying.value = false
    currentIndex.value = -1
  } else if (realIndex < currentIndex.value) {
    currentIndex.value--
  }
  playlist.value.splice(realIndex, 1)
  if (repeatMode.value === 'shuffle') generateShuffleOrder()
  markDirty()
}

// ============ 播放控制 (Electron playSong) ============

async function playSong(index: number) {
  if (useExternalPlayer.value) { await playSongExternal(index); return }
  await playSongInternal(index)
}

async function playSongInternal(index: number) {
  const song = filteredPlaylist.value[index]
  const realIndex = playlist.value.indexOf(song)
  if (realIndex < 0) return
  if (currentIndex.value === realIndex && isPlaying.value) {
    audioRef.value?.pause()
    isPlaying.value = false
    return
  }
  currentIndex.value = realIndex

  // Revoke previous blob URL
  if (currentBlobUrl) revokeBlobUrl(currentBlobUrl)
  currentBlobUrl = null

  try {
    // Step 1: Try convertFileSrc first — instant, no IPC data transfer
    const url = convertFileSrc(song.path)
    if (audioRef.value) {
      audioRef.value.src = url
      audioRef.value.volume = muted.value ? 0 : volume.value
      await audioRef.value.play()
    }
    isPlaying.value = true
  } catch {
    // Step 2: convertFileSrc failed (e.g. asset protocol scope), fall back to Blob URL
    // Cancel any pending error skip from onError
    if (errorTimer) { clearTimeout(errorTimer); errorTimer = undefined }
    errorCount = 0
    _retryingBlob = true
    try {
      const data = await readAudioFile(song.path)
      const blobUrl = URL.createObjectURL(new Blob([new Uint8Array(data)]))
      currentBlobUrl = blobUrl
      if (audioRef.value) {
        audioRef.value.src = blobUrl
        audioRef.value.volume = muted.value ? 0 : volume.value
        await audioRef.value.play()
      }
      isPlaying.value = true
    } catch (e2) {
      isPlaying.value = false
    } finally {
      _retryingBlob = false
    }
  }

  currentLyrics.value = null
  try {
    const meta = await getAudioMetadata(song.path)
    currentLyrics.value = meta.lyrics || null
  } catch (e) { /* ignore */ }
}

async function playSongExternal(index: number) {
  const song = filteredPlaylist.value[index]
  const realIndex = playlist.value.indexOf(song)
  if (realIndex < 0) return
  currentIndex.value = realIndex
  isPlaying.value = true
  try {
    await playWithFfplay(song.path)
  } catch (e: any) {
    isPlaying.value = false
    return
  }
  currentTime.value = 0; duration.value = 0
  try {
    const meta = await getAudioMetadata(song.path)
    currentLyrics.value = meta.lyrics || null
  } catch (e) { currentLyrics.value = null }
}

// ============ 播放按钮 (Electron playBtn) ============

function togglePlay() {
  if (playlist.value.length === 0) return
  if (currentIndex.value < 0) { playSong(0); return }
  if (!audioRef.value) return
  if (isPlaying.value) {
    audioRef.value.pause()
    isPlaying.value = false
  } else {
    audioRef.value.play().then(() => { isPlaying.value = true }).catch(() => {})
  }
}

function prevSong() {
  if (playlist.value.length === 0) return
  let idx: number
  if (repeatMode.value === 'shuffle' && shuffleOrder.value.length > 0) {
    shufflePos.value = shufflePos.value <= 0 ? shuffleOrder.value.length - 1 : shufflePos.value - 1
    idx = shuffleOrder.value[shufflePos.value]
  } else {
    idx = currentIndex.value <= 0 ? playlist.value.length - 1 : currentIndex.value - 1
  }
  playSong(idx)
}

function nextSong() {
  if (playlist.value.length === 0) return
  if (repeatMode.value === 'shuffle' && shuffleOrder.value.length > 0) {
    shufflePos.value++
    if (shufflePos.value >= shuffleOrder.value.length) {
      generateShuffleOrder()
      shufflePos.value = 0
    }
    playSong(shuffleOrder.value[shufflePos.value])
  } else {
    const next = currentIndex.value + 1
    if (next >= playlist.value.length) {
      if (repeatMode.value === 'none') return
      playSong(0)
    } else {
      playSong(next)
    }
  }
}

function cycleRepeat() {
  const modes: string[] = ['none', 'all', 'one', 'shuffle']
  const idx = modes.indexOf(repeatMode.value)
  repeatMode.value = modes[(idx + 1) % modes.length] as any
  if (repeatMode.value === 'shuffle') generateShuffleOrder()
}

function generateShuffleOrder() {
  const n = playlist.value.length
  const arr = Array.from({ length: n }, (_, i) => i)
  for (let i = n - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1)); [arr[i], arr[j]] = [arr[j], arr[i]]
  }
  if (currentIndex.value >= 0 && currentIndex.value < n) {
    const curIdx = arr.indexOf(currentIndex.value)
    if (curIdx > 0) { [arr[0], arr[curIdx]] = [arr[curIdx], arr[0]] }
    shufflePos.value = 0
  } else { shufflePos.value = -1 }
  shuffleOrder.value = arr
}

// ============ Audio Events (Electron timeupdate/loadedmetadata/ended/error) ============

function seek(e: Event) { if (audioRef.value) audioRef.value.currentTime = Number((e.target as HTMLInputElement).value) }
function setVolume(e: Event) { volume.value = Number((e.target as HTMLInputElement).value) / 100; if (audioRef.value) audioRef.value.volume = muted.value ? 0 : volume.value }
function toggleMute() { muted.value = !muted.value; if (audioRef.value) audioRef.value.volume = muted.value ? 0 : volume.value }
function toggleView(v: string) { currentView.value = v as any }
function onTimeUpdate() { if (audioRef.value) currentTime.value = audioRef.value.currentTime }
function onMetadata() { if (audioRef.value) { duration.value = audioRef.value.duration; if (currentIndex.value >= 0) playlist.value[currentIndex.value].duration = duration.value } }

function onEnded() {
  if (repeatMode.value === 'one') { audioRef.value?.play(); return }
  // Electron getNextIndex logic
  if (repeatMode.value === 'shuffle') {
    shufflePos.value++
    if (shufflePos.value < shuffleOrder.value.length) {
      playSongByRealIndex(shuffleOrder.value[shufflePos.value])
      return
    }
    // shuffle done, stop
    isPlaying.value = false
    return
  }
  const next = currentIndex.value + 1
  if (next < playlist.value.length) {
    playSongByRealIndex(next)
  } else if (repeatMode.value === 'all') {
    playSongByRealIndex(0)
  } else {
    isPlaying.value = false
  }
}

let errorCount = 0
let errorTimer: number | undefined

function onError() {
  if (_retryingBlob) return  // suppress skip during Blob URL fallback
  isPlaying.value = false
  const currentIdx = currentIndex.value
  errorCount++
  // Only auto-skip once per song to prevent infinite loop
  if (errorCount === 1 && currentIdx >= 0 && playlist.value.length > 1) {
    if (errorTimer) clearTimeout(errorTimer)
    errorTimer = window.setTimeout(() => {
      // Only skip if we're still on the same broken song
      if (currentIndex.value === currentIdx) {
        nextSong()
      }
      errorCount = 0
    }, 1000)
  } else if (errorCount > 1) {
    // Multiple errors on same song - stop auto-skipping
    errorCount = 0
    if (errorTimer) clearTimeout(errorTimer)
  }
}

function seekProgress(e: MouseEvent) {
  if (!audioRef.value || !duration.value) return
  const el = e.currentTarget as HTMLElement
  const rect = el.getBoundingClientRect()
  audioRef.value.currentTime = ((e.clientX - rect.left) / rect.width) * duration.value
}

function seekToLyric(time: number) {
  if (audioRef.value) audioRef.value.currentTime = time
}

function playSongByRealIndex(realIndex: number) {
  // Find the index in filteredPlaylist that maps to this realIndex
  for (let i = 0; i < filteredPlaylist.value.length; i++) {
    if (playlist.value.indexOf(filteredPlaylist.value[i]) === realIndex) {
      playSong(i)
      return
    }
  }
}

// ============ 浮动菜单 (Electron toggleMenu/openMenu/closeMenu/handleMenuAction) ============

function toggleMenu(e: MouseEvent) {
  menuOpen.value = !menuOpen.value
  if (!menuOpen.value) return
  // Position menu under the button
  const btn = e.currentTarget as HTMLElement
  const rect = btn.getBoundingClientRect()
  if (contextMenuRef.value) {
    contextMenuRef.value.style.top = (rect.bottom + 4) + 'px'
    contextMenuRef.value.style.right = (window.innerWidth - rect.right) + 'px'
  }
}

async function handleMenuAction(action: string) {
  menuOpen.value = false
  switch (action) {
    case 'import':
      try {
        const result = await apiImportPlaylist()
        if (result && result.id) {
          playlistsIndex.value.push({ id: result.id, name: result.name || '导入歌单', is_default: false, created: '', modified: '', count: result.count || 0 })
          await switchPlaylist(result.id)
          showToast('歌单导入成功', 'success')
        }
      } catch (e) { showToast('导入失败', 'error') }
      break
    case 'export-json':
      try {
        await apiExportJson(currentPlaylistId.value)
        showToast('导出 JSON 成功', 'success')
      } catch (e) { showToast('导出失败', 'error') }
      break
    case 'export-m3u':
      try {
        await apiExportM3u(currentPlaylistId.value)
        showToast('导出 M3U 成功', 'success')
      } catch (e) { showToast('导出失败', 'error') }
      break
    case 'rename':
      showRenameModal()
      break
    case 'delete':
      confirmDeletePlaylist(currentPlaylistId.value)
      break
  }
}

// ============ 模态弹窗 (Electron showModal/hideModal/handleModalConfirm) ============

function showCreateModal() {
  modalMode.value = 'create'
  modalTitle.value = '新建歌单'
  modalInput.value = ''
  modalOpen.value = true
  nextTick(() => modalInputRef.value?.focus())
}

function showRenameModal() {
  const current = playlistsIndex.value.find(p => p.id === currentPlaylistId.value)
  if (!current) return
  modalMode.value = 'rename'
  modalTitle.value = '重命名歌单'
  modalInput.value = current.name
  modalOpen.value = true
  nextTick(() => modalInputRef.value?.focus())
}

function confirmDeletePlaylist(playlistId: string) {
  const target = playlistsIndex.value.find(p => p.id === playlistId)
  if (!target) return
  if (target.is_default) {
    modalMode.value = 'deleteConfirm'
    modalTitle.value = '提示'
    modalMessage.value = '默认歌单不能删除'
    pendingDeleteId.value = null
    modalOpen.value = true
    return
  }
  modalMode.value = 'deleteConfirm'
  modalTitle.value = '删除歌单'
  modalMessage.value = `确定要删除歌单「${target.name}」吗？此操作不可撤销。`
  pendingDeleteId.value = playlistId
  modalOpen.value = true
}

function hideModal() {
  modalOpen.value = false
  modalMode.value = null
  pendingDeleteId.value = null
}

async function handleModalConfirm() {
  if (modalMode.value === 'deleteConfirm') {
    if (pendingDeleteId.value) {
      const target = playlistsIndex.value.find(p => p.id === pendingDeleteId.value)
      if (target && !target.is_default) {
        try {
          await apiDeletePlaylist(pendingDeleteId.value)
          playlistsIndex.value = playlistsIndex.value.filter(p => p.id !== pendingDeleteId.value)
          if (pendingDeleteId.value === currentPlaylistId.value) {
            const defaultPl = playlistsIndex.value.find(p => p.is_default) || playlistsIndex.value[0]
            if (defaultPl) await switchPlaylist(defaultPl.id)
          }
          showToast('歌单已删除', 'success')
        } catch (e) { showToast('删除失败', 'error') }
      }
    }
    hideModal()
    return
  }
  const name = modalInput.value.trim()
  if (!name) { modalInputRef.value?.focus(); return }
  if (modalMode.value === 'create') {
    try {
      const [id, _] = await apiCreatePlaylist(name)
      if (id) {
        playlistsIndex.value.push({ id, name, is_default: false, created: '', modified: '', count: 0 })
        await switchPlaylist(id)
        showToast('歌单已创建', 'success')
      }
    } catch (e) { showToast('创建失败', 'error') }
  } else if (modalMode.value === 'rename') {
    try {
      await apiRenamePlaylist(currentPlaylistId.value, name)
      const entry = playlistsIndex.value.find(p => p.id === currentPlaylistId.value)
      if (entry) entry.name = name
      showToast('歌单已重命名', 'success')
    } catch (e) { showToast('重命名失败', 'error') }
  }
  hideModal()
}

// ============ 拖拽排序 (Electron drag-drop) ============

function onDragStart(e: DragEvent) {
  const el = (e.target as HTMLElement).closest('.tab-item') as HTMLElement
  if (!el) { e.preventDefault(); return }
  if ((e.target as HTMLElement).closest('.tab-close')) { e.preventDefault(); return }
  dragId.value = el.dataset.playlistId || null
  el.classList.add('dragging')
  e.dataTransfer!.effectAllowed = 'move'
  e.dataTransfer!.setData('text/plain', dragId.value || '')
}

function onDragOver(e: DragEvent) {
  e.preventDefault()
  e.dataTransfer!.dropEffect = 'move'
  // Auto-scroll
  const container = tabsScrollRef.value?.closest('.tabs-scroll-container') as HTMLElement
  if (container) {
    const cr = container.getBoundingClientRect()
    if (e.clientX < cr.left + 40) container.scrollLeft -= 8
    else if (e.clientX > cr.right - 40) container.scrollLeft += 8
  }
  const target = (e.target as HTMLElement).closest('.tab-item') as HTMLElement
  if (!target || target.dataset.playlistId === dragId.value) return
  const rect = target.getBoundingClientRect()
  const midX = rect.left + rect.width / 2
  dropTarget.value = { id: target.dataset.playlistId!, side: e.clientX < midX ? 'left' : 'right' }
}

function onDragLeave(e: DragEvent) {
  const target = (e.target as HTMLElement).closest('.tab-item')
  if (!target) return
  const rect = target.getBoundingClientRect()
  if (e.clientX < rect.left || e.clientX > rect.right || e.clientY < rect.top || e.clientY > rect.bottom) {
    dropTarget.value = null
  }
}

function onDrop(e: DragEvent) {
  e.preventDefault()
  const target = (e.target as HTMLElement).closest('.tab-item') as HTMLElement
  if (!target || !dragId.value || target.dataset.playlistId === dragId.value) { clearDrag(); return }
  const rect = target.getBoundingClientRect()
  const insertBefore = e.clientX < rect.left + rect.width / 2
  const fromIdx = playlistsIndex.value.findIndex(p => p.id === dragId.value)
  const toId = target.dataset.playlistId!
  if (fromIdx === -1) { clearDrag(); return }
  const [item] = playlistsIndex.value.splice(fromIdx, 1)
  let toIdx = playlistsIndex.value.findIndex(p => p.id === toId)
  if (!insertBefore) toIdx += 1
  playlistsIndex.value.splice(toIdx, 0, item)
  clearDrag()
  try { apiReorderPlaylists(playlistsIndex.value.map(p => p.id)) } catch (e) { /* ignore */ }
}

function onDragEnd() { clearDrag() }

function clearDrag() {
  dragId.value = null
  dropTarget.value = null
  document.querySelectorAll('.tab-item.dragging').forEach(el => el.classList.remove('dragging'))
}

// ============ 自动保存 (Electron markDirty/scheduleAutosave/saveBeforeUnload) ============

function markDirty() {
  dirty.value = true
  scheduleAutosave()
}

function scheduleAutosave() {
  if (autosaveTimer) clearTimeout(autosaveTimer)
  autosaveTimer = window.setTimeout(() => saveCurrentPlaylist(), 2000)
}

async function saveCurrentPlaylist() {
  if (!currentPlaylistId.value) return
  try {
    const entry = playlistsIndex.value.find(p => p.id === currentPlaylistId.value)
    const songs = playlist.value.map(s => ({ id: s.id || `song-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, path: s.path, name: s.name, duration: s.duration || 0 }))
    await savePlaylist({ id: currentPlaylistId.value, name: entry?.name || '未命名歌单', created: '', modified: '', songs })
    dirty.value = false
    if (entry) entry.count = songs.length
  } catch (e) { /* ignore */ }
}

// Watch for changes to trigger auto-save
watch(playlist, () => markDirty(), { deep: true })

// 歌词滚动 — 当前行保持居中
watch(currentLyricIndex, (idx) => {
  if (idx < 0 || !lyricsContentRef.value) return
  const container = lyricsContentRef.value
  const lines = container.querySelectorAll('.lyrics-line')
  if (!lines[idx]) return
  const lineEl = lines[idx] as HTMLElement
  const containerHeight = container.clientHeight
  const lineOffset = lineEl.offsetTop
  const lineHeight = lineEl.offsetHeight
  container.scrollTop = lineOffset - containerHeight / 2 + lineHeight / 2
})
</script>
