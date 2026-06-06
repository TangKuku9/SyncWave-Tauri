/**
 * SVG 图标库 - 从 SyncWave utils.js 移植
 * 所有图标使用 currentColor，可通过 CSS color 属性控制颜色
 */
export function svg(paths: string, size = 24): string {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="${size}" height="${size}" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${paths}</svg>`
}

export function svgFill(paths: string, size = 24): string {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="${size}" height="${size}" fill="currentColor" stroke="none">${paths}</svg>`
}

export const icons: Record<string, (size?: number) => string> = {
  // 侧边栏/页面图标
  musicNote: (s = 24) => svg(`<path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/>`, s),
  headphones: (s = 24) => svg(`<path d="M3 18v-6a9 9 0 0 1 18 0v6"/><path d="M21 19a2 2 0 0 1-2 2h-1a2 2 0 0 1-2-2v-3a2 2 0 0 1 2-2h3z"/><path d="M3 19a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2v-3a2 2 0 0 0-2-2H3z"/>`, s),
  videoCamera: (s = 24) => svg(`<rect x="2" y="4" width="15" height="16" rx="2"/><path d="M17 8l5-3v14l-5-3z"/>`, s),
  arrowsRotate: (s = 24) => svg(`<path d="M21 2v6h-6"/><path d="M3 12a9 9 0 0 1 15-6.7L21 8"/><path d="M3 22v-6h6"/><path d="M21 12a9 9 0 0 1-15 6.7L3 16"/>`, s),
  speaker: (s = 24) => svg(`<path d="M11 5L6 9H2v6h4l5 4V5z"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>`, s),
  scissors: (s = 24) => svg(`<circle cx="6" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><line x1="20" y1="4" x2="8.12" y2="15.88"/><line x1="14.47" y1="14.48" x2="20" y2="20"/><line x1="8.12" y1="8.12" x2="12" y2="12"/>`, s),
  compress: (s = 24) => svg(`<path d="M4 14h6v6"/><path d="M20 10h-6V4"/><path d="M14 10l7-7"/><path d="M3 21l7-7"/>`, s),
  image: (s = 24) => svg(`<rect x="3" y="3" width="18" height="18" rx="2"/><circle cx="8.5" cy="8.5" r="1.5"/><path d="M21 15l-5-5L5 21"/>`, s),
  plus: (s = 24) => svg(`<line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>`, s),
  folder: (s = 24) => svg(`<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>`, s),
  videoFile: (s = 24) => svg(`<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M10 11l4 3-4 3z" fill="currentColor" stroke="none"/>`, s),
  wrench: (s = 24) => svg(`<path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>`, s),

  // 播放器控制图标
  play: (s = 24) => svgFill(`<path d="M6 3v18l15-9z"/>`, s),
  pause: (s = 24) => svgFill(`<rect x="5" y="3" width="5" height="18" rx="1"/><rect x="14" y="3" width="5" height="18" rx="1"/>`, s),
  prev: (s = 24) => svgFill(`<path d="M19 20L9 12l10-8v16zM5 4h3v16H5z"/>`, s),
  next: (s = 24) => svgFill(`<path d="M5 4l10 8-10 8V4zM16 4h3v16h-3z"/>`, s),
  repeatOff: (s = 24) => svg(`<path d="M17 1l4 4-4 4"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><path d="M7 23l-4-4 4-4"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/><line x1="2" y1="2" x2="22" y2="22"/>`, s),
  repeat: (s = 24) => svg(`<path d="M17 1l4 4-4 4"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><path d="M7 23l-4-4 4-4"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>`, s),
  repeatOne: (s = 24) => svg(`<path d="M17 1l4 4-4 4"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><path d="M7 23l-4-4 4-4"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/><text x="12" y="15" text-anchor="middle" fill="currentColor" stroke="none" font-size="7" font-weight="bold">1</text>`, s),
  shuffle: (s = 24) => svg(`<path d="M16 3h5v5"/><path d="M4 20L21 3"/><path d="M21 16v5h-5"/><path d="M15 15l6 6"/><path d="M4 4l5 5"/>`, s),
  list: (s = 24) => svg(`<line x1="9" y1="6" x2="21" y2="6"/><line x1="9" y1="12" x2="21" y2="12"/><line x1="9" y1="18" x2="21" y2="18"/><line x1="4" y1="6" x2="4.01" y2="6"/><line x1="4" y1="12" x2="4.01" y2="12"/><line x1="4" y1="18" x2="4.01" y2="18"/>`, s),
  lyrics: (s = 24) => svgFill(`<path d="M4 2a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H4zm3 5h10v1.5H7V7zm0 3.5h10V12H7v-1.5zm0 3.5h7v1.5H7V14zm10 3a2.5 2.5 0 1 1 0-5 2.5 2.5 0 0 1 0 5z"/>`, s),
  fileText: (s = 24) => svg(`<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="14" y2="17"/><line x1="8" y1="9" x2="10" y2="9"/>`, s),

  // 音量图标
  volumeHigh: (s = 24) => svg(`<path d="M11 5L6 9H2v6h4l5 4V5z"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>`, s),
  volumeLow: (s = 24) => svg(`<path d="M11 5L6 9H2v6h4l5 4V5z"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>`, s),
  volumeMute: (s = 24) => svg(`<path d="M11 5L6 9H2v6h4l5 4V5z"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/>`, s),

  // 歌单管理图标
  playlist: (s = 24) => svg(`<line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><path d="M3 6h.01"/><path d="M3 12h.01"/><path d="M3 18h.01"/>`, s),
  moreVertical: (s = 24) => svgFill(`<circle cx="12" cy="5" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="12" cy="19" r="1.5"/>`, s),
  fileImport: (s = 24) => svg(`<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M12 18v-6"/><path d="M9 15l3 3 3-3"/>`, s),
  fileExport: (s = 24) => svg(`<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M12 12v6"/><path d="M9 15l3-3 3 3"/>`, s),
  rename: (s = 24) => svg(`<path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>`, s),
  trash: (s = 24) => svg(`<path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/>`, s),

  // 状态图标
  check: (s = 24) => svg(`<path d="M20 6L9 17l-5-5"/>`, s),
  cross: (s = 24) => svg(`<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>`, s),
  warning: (s = 24) => svg(`<path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>`, s),
  close: (s = 24) => svg(`<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>`, s),

  // 窗口控制
  minimize: (s = 14) => svg(`<line x1="2" y1="12" x2="22" y2="12"/>`, s),
  maximize: (s = 14) => svg(`<rect x="3" y="3" width="18" height="18" rx="2"/>`, s),
  restore: (s = 14) => svg(`<rect x="3" y="5" width="14" height="14" rx="2"/><path d="M7 5V3a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2h-2"/>`, s),
}
