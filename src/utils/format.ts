/**
 * 格式化工具函数 - 从 SyncWave utils.js 移植
 */

/** 格式化时间（秒 -> mm:ss） */
export function formatTime(seconds: number): string {
  if (isNaN(seconds) || !isFinite(seconds)) return '0:00'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

/** 从文件路径提取文件名 */
export function extractFileName(filePath: string): string {
  return filePath.split('\\').pop()?.split('/').pop() || filePath
}

/** 从文件名提取歌曲名（去除扩展名） */
export function extractSongName(filePath: string): string {
  const fileName = extractFileName(filePath)
  return fileName.replace(/\.[^/.]+$/, '')
}

/** Toast 通知 */
export function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success', duration = 2500) {
  let container = document.querySelector('.toast-container')
  if (!container) {
    container = document.createElement('div')
    container.className = 'toast-container'
    document.body.appendChild(container)
  }

  const toast = document.createElement('div')
  toast.className = `toast toast-${type}`
  toast.textContent = message
  container.appendChild(toast)

  setTimeout(() => {
    toast.classList.add('toast-hiding')
    setTimeout(() => {
      if (toast.parentNode) toast.parentNode.removeChild(toast)
      if (container.children.length === 0 && container.parentNode) container.parentNode.removeChild(container)
    }, 300)
  }, duration)
}
