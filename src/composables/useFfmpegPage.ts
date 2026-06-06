/**
 * FFmpeg 页面通用逻辑 composable
 * 对应 SyncWave 的 ffmpeg-page-factory.js
 */
import { ref } from 'vue'
import { selectMediaFile, selectFolder } from '../utils/api'
import { onEvent, type FfmpegResult, type ProgressPayload, type LogPayload } from '../utils/api'
import type { LogEntry } from '../components/LogPanel.vue'

export interface FfmpegPageOptions {
  channelPrefix: string
  selectFileApi?: () => Promise<string | null>
  getParams: () => Record<string, any>
  buildArgs: (filePath: string, params: Record<string, any>) => { args: Record<string, any>; outputDir?: string }
  runApi: (args: Record<string, any>) => Promise<FfmpegResult>
}

export function useFfmpegPage(options: FfmpegPageOptions) {
  const filePath = ref<string | null>(null)
  const fileName = ref('')
  const outputDir = ref('')
  const processing = ref(false)
  const progress = ref(0)
  const progressText = ref('')
  const logs = ref<LogEntry[]>([])

  function addLog(message: string, type = 'info') {
    logs.value.push({
      time: new Date().toLocaleTimeString(),
      message,
      type,
    })
  }

  async function selectFile() {
    const selectFn = options.selectFileApi || selectMediaFile
    const file = await selectFn()
    if (file) {
      filePath.value = file
      fileName.value = file.split('\\').pop()?.split('/').pop() || file
      addLog(`已选择: ${fileName.value}`)
    }
  }

  async function selectOutputDir() {
    const dir = await selectFolder()
    if (dir) {
      outputDir.value = dir
    }
  }

  async function startProcess() {
    if (!filePath.value || processing.value) return

    processing.value = true
    progress.value = 0
    progressText.value = ''
    logs.value = []

    addLog('开始处理...')

    // Listen for events
    const unlistenProgress = await onEvent<ProgressPayload>(
      `${options.channelPrefix}-progress`,
      (data) => {
        if (data.percent !== undefined && data.percent !== null) {
          progress.value = Math.min(99, data.percent)
          progressText.value = `${Math.round(data.percent)}%`
        }
      }
    )

    const unlistenLog = await onEvent<LogPayload>(
      `${options.channelPrefix}-log`,
      (data) => {
        addLog(data.message, data.type)
      }
    )

    try {
      const params = options.getParams()
      const { args } = options.buildArgs(filePath.value, params)
      if (outputDir.value) {
        args.outputDir = outputDir.value
      }

      const result = await options.runApi(args)

      if (result.success) {
        progress.value = 100
        progressText.value = '完成'
        addLog('处理完成', 'success')
      } else {
        progress.value = 0
        progressText.value = '失败'
        addLog(`失败: ${result.error}`, 'error')
      }
    } catch (e: any) {
      addLog(`错误: ${e.message || e}`, 'error')
      progress.value = 0
      progressText.value = '失败'
    } finally {
      processing.value = false
      unlistenProgress()
      unlistenLog()
    }
  }

  return {
    filePath,
    fileName,
    outputDir,
    processing,
    progress,
    progressText,
    logs,
    selectFile,
    selectOutputDir,
    startProcess,
    addLog,
  }
}
