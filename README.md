**SyncWave**

一款由 Electron 项目迁移并使用 Tauri 重构的桌面多媒体工具，目标是保留原有功能的同时大幅度优化性能、减小体积并提升安全性与启动速度。

**亮点**
- **重构说明**: 原 Electron 框架项目 `SyncWave` 已使用 Tauri（Rust + Web 前端）完成重构，获得更低的资源占用与更小的发布体积。
- **前端**: 使用 Vue + Vite 构建，代码位于 [src/main.js](src/main.js#L1) 及 [src](src)
- **后台/原生层**: 使用 Rust（位于 [src-tauri/src/main.rs](src-tauri/src/main.rs#L1)）承载与系统交互、FFmpeg 调用等高性能任务。

**主要特性**
- 多功能视频处理（合并、裁剪、压缩、提取音频、转码等）
- 集成 FFmpeg，支持本地二进制或内置平台二进制（见 `src-tauri/binaries/`）
- 更快的启动速度与更低的内存占用（相比传统 Electron 架构）
- 更小的安装包体积与更强的本地能力（通过 Rust 原生扩展）

**项目结构（简要）**
- **前端**: [src](src) — Vue 组件、页面、样式与前端逻辑
- **Tauri / 原生**: [src-tauri](src-tauri) — Rust 原生代码、FFmpeg 接口、打包配置
- **打包配置**: [package.json](package.json#L1)，包含前端与打包脚本

**快速上手**
1. 环境要求：
	- Node.js（推荐 16+ 或项目要求的版本）
	- Rust toolchain（用于编译 Tauri 原生部分）
	- 平台依赖：根据目标平台，可能需要额外的原生依赖（详见 Tauri 官方文档）
2. 安装依赖：
	- `npm install`（或 `pnpm/npm`）
3. 开发运行：
	- 本地开发（热重载）：`npm run tauri dev` 或项目中定义的等效脚本
4. 打包发布：
	- 使用 Tauri 打包：`npm run tauri build`（或项目中定义的打包脚本）

备注：仓库内已包含用于不同平台的二进制/配置目录（`src-tauri/binaries/`），请在打包前确认所需 FFmpeg 二进制是否齐全或按需替换。

**性能与迁移优势说明**
- Tauri 将 UI 与应用的原生宿主分离，前端仍使用轻量的 Web 技术（Vite + Vue），而系统交互与资源密集型任务交由 Rust 处理：
  - 降低了常驻内存与运行时开销
  - 缩减最终安装包体积
  - 提升应用启动速度与稳定性

**贡献**
- 欢迎提交 Issue 与 Pull Request。若要参与开发，请先阅读代码并在 PR 中说明变更点与测试步骤。

**许可证**
- 请查看仓库内的 `LICENSE` 文件以获取许可信息。

谢谢使用 SyncWave — 如果需要，我可以帮你补充更详细的构建脚本说明、Windows 打包步骤或在 README 中加入演示 GIF/截图。
# Tauri + Vue 3

This template should help get you started developing with Tauri + Vue 3 in Vite. The template uses Vue 3 `<script setup>` SFCs, check out the [script setup docs](https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup) to learn more.

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
