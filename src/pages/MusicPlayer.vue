<template>
  <div class="page-container active music-player-page">
    <div class="content-header">
      <h2>音乐播放器</h2>
      <span class="badge">player</span>
    </div>

    <!-- 顶部控制栏 -->
    <div class="music-controls-top">
      <button class="add-music-btn" @click="addMusic">
        <Icon name="plus" :size="16" /> <span>添加音乐</span>
      </button>
      <button class="add-music-btn" style="background:var(--bg-tertiary);color:var(--text-secondary);" @click="scanDir">
        <Icon name="folder" :size="16" /> <span>扫描目录</span>
      </button>
      <div class="search-box">
        <input type="text" v-model="searchQuery" placeholder="搜索歌曲..." />
      </div>
    </div>

    <!-- 播放列表/歌词视图 -->
    <div class="playlist-views">
      <!-- 播放列表 -->
      <div class="playlist-view" :class="{ active: currentView === 'playlist' }">
        <div class="playlist-header">
          <span class="playlist-title">播放列表</span>
          <span class="playlist-count">{{ playlist.length }} 首</span>
        </div>
        <div class="playlist-list">
          <div class="playlist-empty" v-if="playlist.length === 0">
            <div class="empty-icon"><Icon name="headphones" :size="48" /></div>
            <div class="empty-text">暂无音乐，请添加音乐文件</div>
          </div>
          <div
            v-for="(song, i) in filteredPlaylist" :key="song.id"
            class="playlist-item" :class="{ playing: currentIndex === i && isPlaying }"
            @click="playSong(i)"
          >
            <span class="playlist-item-index">{{ i + 1 }}</span>
            <span class="playlist-item-icon"><Icon :name="currentIndex === i && isPlaying ? 'pause' : 'play'" :size="16" /></span>
            <div class="playlist-item-info">
              <div class="playlist-item-name">{{ song.name }}</div>
            </div>
            <span class="playlist-item-duration">{{ song.duration ? formatTime(song.duration) : '' }}</span>
            <button class="playlist-item-remove" @click.stop="removeSong(i)"><Icon name="close" :size="14" /></button>
          </div>
        </div>
      </div>

      <!-- 歌词视图 -->
      <div class="lyrics-view" :class="{ active: currentView === 'lyrics' }">
        <div class="lyrics-content" v-if="currentLyrics">
          <div
            v-for="(line, i) in parsedLyrics" :key="i"
            class="lyrics-line" :class="{ highlight: currentLyricIndex === i }"
          >{{ line.text }}</div>
        </div>
        <div class="lyrics-empty" v-else>暂无歌词</div>
      </div>
    </div>

    <!-- 播放器控制区 -->
    <div class="player-controls">
      <div class="now-playing">
        <div class="album-art"><Icon name="musicNote" :size="28" /></div>
        <div class="song-info">
          <div class="song-title">{{ currentSong?.name || '未播放' }}</div>
          <div class="song-artist"> </div>
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
        <button class="control-btn" @click="cycleRepeat" :title="repeatLabel">
          <Icon :name="repeatIcon" :size="20" />
        </button>
        <button class="control-btn" @click="toggleView" title="播放列表">
          <Icon :name="currentView === 'playlist' ? 'lyrics' : 'list'" :size="20" />
        </button>
      </div>
    </div>

    <!-- 隐藏的 audio 元素 -->
    <audio ref="audioRef" @timeupdate="onTimeUpdate" @loadedmetadata="onMetadata" @ended="onEnded" @error="onError"></audio>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { selectMusicFiles, getMusicUrl, getAudioMetadata, loadPlaylistsIndex, loadPlaylist, savePlaylist, selectFolder, scanDirectory } from '../utils/api'
import type { Song, Playlist, PlaylistIndexEntry } from '../utils/api'
import { formatTime } from '../utils/format'
import Icon from '../components/Icon.vue'

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
const currentPlaylistId = ref('default')

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

onMounted(async () => {
  try {
    const index = await loadPlaylistsIndex()
    if (index.length > 0) {
      currentPlaylistId.value = index[0].id
      const pl = await loadPlaylist(currentPlaylistId.value)
      playlist.value = pl.songs || []
    }
  } catch (e) { /* ignore */ }
})

async function addMusic() {
  const files = await selectMusicFiles()
  if (!files || files.length === 0) return
  for (const f of files) {
    const name = f.split('\\').pop()?.split('/').pop()?.replace(/\.[^/.]+$/, '') || f
    playlist.value.push({ id: `song-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, path: f, name, duration: 0 })
  }
}

function removeSong(index: number) {
  const realIndex = playlist.value.indexOf(filteredPlaylist.value[index])
  if (realIndex >= 0) {
    playlist.value.splice(realIndex, 1)
    if (currentIndex.value === realIndex) { currentIndex.value = -1; isPlaying.value = false }
    else if (currentIndex.value > realIndex) currentIndex.value--
  }
}

async function playSong(index: number) {
  const song = filteredPlaylist.value[index]
  const realIndex = playlist.value.indexOf(song)
  if (realIndex < 0) return

  if (currentIndex.value === realIndex && isPlaying.value) {
    audioRef.value?.pause()
    isPlaying.value = false
    return
  }

  currentIndex.value = realIndex
  const url = await getMusicUrl(song.path)
  if (audioRef.value) {
    audioRef.value.src = url
    audioRef.value.volume = muted.value ? 0 : volume.value
    audioRef.value.play().then(() => { isPlaying.value = true }).catch(() => {})
  }

  // Load lyrics
  try {
    const meta = await getAudioMetadata(song.path)
    currentLyrics.value = meta.lyrics || null
    if (meta.size > 0 && !song.duration) {
      // duration will be set by loadedmetadata
    }
  } catch (e) { currentLyrics.value = null }
}

function togglePlay() {
  if (!audioRef.value) return
  if (isPlaying.value) { audioRef.value.pause(); isPlaying.value = false }
  else { audioRef.value.play().then(() => { isPlaying.value = true }).catch(() => {}) }
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
  let idx: number
  if (repeatMode.value === 'shuffle' && shuffleOrder.value.length > 0) {
    shufflePos.value++
    if (shufflePos.value >= shuffleOrder.value.length) {
      generateShuffleOrder()
      shufflePos.value = 0
    }
    idx = shuffleOrder.value[shufflePos.value]
  } else {
    idx = currentIndex.value >= playlist.value.length - 1 ? 0 : currentIndex.value + 1
  }
  playSong(idx)
}

function cycleRepeat() {
  const modes: string[] = ['none', 'all', 'one', 'shuffle']
  const idx = modes.indexOf(repeatMode.value)
  repeatMode.value = modes[(idx + 1) % modes.length] as any
  if (repeatMode.value === 'shuffle') {
    generateShuffleOrder()
  }
}

/** Fisher-Yates shuffle */
function generateShuffleOrder() {
  const n = playlist.value.length
  const arr = Array.from({ length: n }, (_, i) => i)
  for (let i = n - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [arr[i], arr[j]] = [arr[j], arr[i]]
  }
  // ensure current song is first in shuffle
  if (currentIndex.value >= 0 && currentIndex.value < n) {
    const curIdx = arr.indexOf(currentIndex.value)
    if (curIdx > 0) {
      [arr[0], arr[curIdx]] = [arr[curIdx], arr[0]]
    }
    shufflePos.value = 0
  } else {
    shufflePos.value = -1
  }
  shuffleOrder.value = arr
}

function seek(e: Event) { if (audioRef.value) audioRef.value.currentTime = Number((e.target as HTMLInputElement).value) }
function setVolume(e: Event) { volume.value = Number((e.target as HTMLInputElement).value) / 100; if (audioRef.value) audioRef.value.volume = muted.value ? 0 : volume.value }
function toggleMute() { muted.value = !muted.value; if (audioRef.value) audioRef.value.volume = muted.value ? 0 : volume.value }
function toggleView() { currentView.value = currentView.value === 'playlist' ? 'lyrics' : 'playlist' }

function onTimeUpdate() { if (audioRef.value) currentTime.value = audioRef.value.currentTime }
function onMetadata() { if (audioRef.value) { duration.value = audioRef.value.duration; if (currentIndex.value >= 0) playlist.value[currentIndex.value].duration = duration.value } }
function onEnded() {
  if (repeatMode.value === 'one') { audioRef.value?.play(); return }
  nextSong()
}
function onError() {
  isPlaying.value = false
  // auto-skip to next after error
  if (currentIndex.value >= 0 && playlist.value.length > 1) {
    setTimeout(() => nextSong(), 500)
  }
}

/** Click on progress bar to seek */
function seekProgress(e: MouseEvent) {
  if (!audioRef.value || !duration.value) return
  const el = e.currentTarget as HTMLElement
  const rect = el.getBoundingClientRect()
  const ratio = (e.clientX - rect.left) / rect.width
  audioRef.value.currentTime = ratio * duration.value
}

/** Add a directory scan function */
async function scanDir() {
  try {
    const dir = await selectFolder()
    if (!dir) return
    const files = await scanDirectory(dir)
    if (files.length === 0) return
    for (const f of files) {
      const name = f.split('\\').pop()?.split('/').pop()?.replace(/\.[^/.]+$/, '') || f
      if (!playlist.value.find(s => s.path === f)) {
        playlist.value.push({ id: `song-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`, path: f, name, duration: 0 })
      }
    }
  } catch (e) { /* ignore */ }
}

// Auto-save playlist
let saveTimer: number | undefined
watch(playlist, () => {
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = window.setTimeout(async () => {
    if (currentPlaylistId.value) {
      await savePlaylist({ id: currentPlaylistId.value, name: '', created: '', modified: '', songs: playlist.value })
    }
  }, 2000)
}, { deep: true })
</script>
