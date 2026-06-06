/**
 * Tauri API 桥接 - 替代 Electron preload.js 的 window.portalAPI
 */
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

// ============ 类型定义 ============

export interface NcmConvertResult {
  file: string
  success: boolean
  error?: string
}

export interface NcmConvertProgress {
  index: number
  total: number
  file: string
  status: string
}

export interface NcmConvertLog {
  time: string
  message: string
  type: string
}

export interface FfmpegResult {
  success: boolean
  output_path?: string
  error?: string
}

export interface ProgressPayload {
  status: string
  percent?: number
  current_time?: number
  total_duration?: number
  index?: number
  total?: number
  file?: string
}

export interface LogPayload {
  time: string
  message: string
  type: string
}

export interface PlaylistIndexEntry {
  id: string
  name: string
  is_default: boolean
  created: string
  modified: string
  count: number
}

export interface Song {
  id: string
  path: string
  name: string
  duration: number
}

export interface Playlist {
  id: string
  name: string
  created: string
  modified: string
  songs: Song[]
}

export interface AudioMetadata {
  path: string
  name: string
  size: number
  exists: boolean
  lyrics?: string
  error?: string
}

export interface OperationResult {
  success: boolean
  error?: string
  file_path?: string
}

export interface ImportResult {
  id?: string
  name?: string
  count: number
  failed: number
  error?: string
}

// ============ 文件选择 ============

export const selectFiles = () => invoke<string[]>('select_files')
export const selectFolder = () => invoke<string | null>('select_folder')
export const selectVideoFile = () => invoke<string | null>('select_video_file')
export const selectAudioFile = () => invoke<string | null>('select_audio_file')
export const selectMediaFile = () => invoke<string | null>('select_media_file')

// ============ NCM 转换 ============

export const convertNcm = (files: string[], outputDir?: string, convertType?: string) =>
  invoke<NcmConvertResult[]>('convert_ncm', { files, outputDir, convertType })

// ============ 音乐播放器 ============

export const selectMusicFiles = () => invoke<string[]>('select_music_files')
export const scanDirectory = (dirPath: string) => invoke<string[]>('scan_directory', { dirPath })
export const getMusicUrl = (filePath: string) => invoke<string>('get_music_url', { filePath })
export const getAudioMetadata = (filePath: string) => invoke<AudioMetadata>('get_audio_metadata', { filePath })

// ============ FFmpeg 工具 ============

export const mergeAv = (videoPath: string, audioPath: string, outputDir?: string) =>
  invoke<FfmpegResult>('merge_av', { videoPath, audioPath, outputDir })

export const formatConvert = (inputPath: string, outputFormat: string, outputDir?: string) =>
  invoke<FfmpegResult>('format_convert', { inputPath, outputFormat, outputDir })

export const extractAudio = (inputPath: string, audioFormat: string, outputDir?: string) =>
  invoke<FfmpegResult>('extract_audio', { inputPath, audioFormat, outputDir })

export const clipVideo = (inputPath: string, startTime: string, endTime: string, outputDir?: string) =>
  invoke<FfmpegResult>('clip_video', { inputPath, startTime, endTime, outputDir })

export const compressVideo = (inputPath: string, crf: string, scale?: string, outputDir?: string) =>
  invoke<FfmpegResult>('compress_video', { inputPath, crf, scale, outputDir })

export const videoToGif = (inputPath: string, fps?: number, width?: number, aspectRatio?: string, startTime?: string, endTime?: string, outputDir?: string) =>
  invoke<FfmpegResult>('video_to_gif', { inputPath, fps, width, aspectRatio, startTime, endTime, outputDir })

// ============ 歌单管理 ============

export const loadPlaylistsIndex = () => invoke<PlaylistIndexEntry[]>('load_playlists_index_cmd')
export const loadPlaylist = (playlistId: string) => invoke<Playlist>('load_playlist_cmd', { playlistId })
export const savePlaylist = (playlist: Playlist) => invoke<OperationResult>('save_playlist_cmd', { playlist })
export const createPlaylist = (name: string) => invoke<[string | null, string | null]>('create_playlist_cmd', { name })
export const deletePlaylist = (playlistId: string) => invoke<OperationResult>('delete_playlist_cmd', { playlistId })
export const renamePlaylist = (playlistId: string, newName: string) => invoke<OperationResult>('rename_playlist_cmd', { playlistId, newName })
export const reorderPlaylists = (orderedIds: string[]) => invoke<OperationResult>('reorder_playlists_cmd', { orderedIds })
export const importPlaylist = () => invoke<ImportResult>('import_playlist_cmd')
export const exportPlaylistJson = (playlistId: string) => invoke<OperationResult>('export_playlist_json_cmd', { playlistId })
export const exportPlaylistM3u = (playlistId: string) => invoke<OperationResult>('export_playlist_m3u_cmd', { playlistId })

// ============ 事件监听 ============

export function onEvent<T>(event: string, callback: (payload: T) => void): Promise<UnlistenFn> {
  return listen<T>(event, (e) => callback(e.payload))
}
