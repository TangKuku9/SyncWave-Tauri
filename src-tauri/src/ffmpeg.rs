use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FfmpegResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProgressPayload {
    pub status: String,
    pub percent: Option<f64>,
    pub current_time: Option<f64>,
    pub total_duration: Option<f64>,
    pub index: Option<usize>,
    pub total: Option<usize>,
    pub file: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogPayload {
    pub time: String,
    pub message: String,
    #[serde(rename = "type")]
    pub log_type: String,
}

fn now_string() -> String {
    let now = chrono::Local::now();
    now.format("%H:%M:%S").to_string()
}

/// Get the path to a sidecar binary, works in both dev and release
fn get_sidecar_path(app: &AppHandle, name: &str) -> PathBuf {
    let exe_name = name.split('/').last().unwrap_or(name);
    let full_exe_name = format!("{}.exe", exe_name);

    // Try resource directory first (release mode)
    let resource_dir = app
        .path()
        .resource_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap());
    let release_path = resource_dir.join(&full_exe_name);
    if release_path.exists() {
        return release_path;
    }

    // Try next to the executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let sidecar_path = exe_dir.join(&full_exe_name);
            if sidecar_path.exists() {
                return sidecar_path;
            }
        }
    }

    // Dev mode: use CARGO_MANIFEST_DIR (set by cargo, works regardless of CWD)
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let dev_path = PathBuf::from(&manifest_dir)
            .join("binaries")
            .join(format!("{}-x86_64-pc-windows-msvc.exe", exe_name));
        if dev_path.exists() {
            return dev_path;
        }
        let dev_path2 = PathBuf::from(&manifest_dir).join("binaries").join(&full_exe_name);
        if dev_path2.exists() {
            return dev_path2;
        }
    }

    // Fallback: relative path from CWD
    let dev_path = PathBuf::from("src-tauri/binaries").join(format!(
        "{}-x86_64-pc-windows-msvc.exe", exe_name
    ));
    if dev_path.exists() {
        return dev_path;
    }

    // Final fallback: just the name (let OS PATH resolve it)
    PathBuf::from(&full_exe_name)
}

pub fn get_ffmpeg_path(app: &AppHandle) -> PathBuf {
    get_sidecar_path(app, "binaries/ffmpeg")
}

pub fn get_ffprobe_path(app: &AppHandle) -> PathBuf {
    get_sidecar_path(app, "binaries/ffprobe")
}

pub fn get_unlock_music_path(app: &AppHandle) -> PathBuf {
    get_sidecar_path(app, "binaries/unlock-music")
}

pub fn get_ncmdump_path(app: &AppHandle) -> PathBuf {
    get_sidecar_path(app, "binaries/ncmdump")
}

/// Get media duration in seconds using ffprobe
pub fn get_media_duration(app: &AppHandle, file_path: &str) -> Option<f64> {
    let ffprobe = get_ffprobe_path(app);
    if !ffprobe.exists() {
        return None;
    }

    let output = StdCommand::new(&ffprobe)
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            file_path,
        ])
        .output()
        .ok()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim().parse::<f64>().ok()
}

/// Run ffmpeg with real-time progress reporting
pub fn run_ffmpeg(
    app: &AppHandle,
    args: &[String],
    channel_prefix: &str,
    output_file_path: &str,
    input_file_path: Option<&str>,
) -> FfmpegResult {
    let ffmpeg = get_ffmpeg_path(app);

    if !ffmpeg.exists() {
        let _ = app.emit(
            &format!("{}-log", channel_prefix),
            LogPayload {
                time: now_string(),
                message: "未找到 ffmpeg.exe".to_string(),
                log_type: "error".to_string(),
            },
        );
        let _ = app.emit(
            &format!("{}-complete", channel_prefix),
            FfmpegResult {
                success: false,
                output_path: None,
                error: Some("ffmpeg.exe not found".to_string()),
            },
        );
        return FfmpegResult {
            success: false,
            output_path: None,
            error: Some("ffmpeg.exe not found".to_string()),
        };
    }

    // Get total duration for progress calculation
    let total_duration = input_file_path
        .and_then(|p| get_media_duration(app, p));

    let _ = app.emit(
        &format!("{}-progress", channel_prefix),
        ProgressPayload {
            status: "processing".to_string(),
            percent: Some(0.0),
            current_time: None,
            total_duration,
            index: None,
            total: None,
            file: None,
        },
    );

    // Run ffmpeg
    let result = StdCommand::new(&ffmpeg)
        .args(args)
        .output();

    match result {
        Ok(output) => {
            if output.status.success() {
                let output_path = Path::new(output_file_path);
                if output_path.exists() {
                    let size = output_path.metadata().map(|m| m.len()).unwrap_or(0);
                    let size_mb = (size as f64) / (1024.0 * 1024.0);

                    let _ = app.emit(
                        &format!("{}-progress", channel_prefix),
                        ProgressPayload {
                            status: "done".to_string(),
                            percent: Some(100.0),
                            current_time: None,
                            total_duration,
                            index: None,
                            total: None,
                            file: None,
                        },
                    );
                    let _ = app.emit(
                        &format!("{}-log", channel_prefix),
                        LogPayload {
                            time: now_string(),
                            message: format!(
                                "完成: {} ({:.1} MB)",
                                output_path.file_name().unwrap_or_default().to_string_lossy(),
                                size_mb
                            ),
                            log_type: "success".to_string(),
                        },
                    );
                    let _ = app.emit(
                        &format!("{}-complete", channel_prefix),
                        FfmpegResult {
                            success: true,
                            output_path: Some(output_file_path.to_string()),
                            error: None,
                        },
                    );
                    FfmpegResult {
                        success: true,
                        output_path: Some(output_file_path.to_string()),
                        error: None,
                    }
                } else {
                    let _ = app.emit(
                        &format!("{}-progress", channel_prefix),
                        ProgressPayload {
                            status: "error".to_string(),
                            percent: None,
                            current_time: None,
                            total_duration,
                            index: None,
                            total: None,
                            file: None,
                        },
                    );
                    let _ = app.emit(
                        &format!("{}-log", channel_prefix),
                        LogPayload {
                            time: now_string(),
                            message: "输出文件未生成".to_string(),
                            log_type: "error".to_string(),
                        },
                    );
                    let _ = app.emit(
                        &format!("{}-complete", channel_prefix),
                        FfmpegResult {
                            success: false,
                            output_path: None,
                            error: Some("输出文件未生成".to_string()),
                        },
                    );
                    FfmpegResult {
                        success: false,
                        output_path: None,
                        error: Some("输出文件未生成".to_string()),
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let error_msg = if stderr.len() > 200 {
                    format!("{}...", &stderr[..200])
                } else {
                    stderr.to_string()
                };
                let _ = app.emit(
                    &format!("{}-progress", channel_prefix),
                    ProgressPayload {
                        status: "error".to_string(),
                        percent: None,
                        current_time: None,
                        total_duration,
                        index: None,
                        total: None,
                        file: None,
                    },
                );
                let _ = app.emit(
                    &format!("{}-log", channel_prefix),
                    LogPayload {
                        time: now_string(),
                        message: format!("失败: {}", error_msg),
                        log_type: "error".to_string(),
                    },
                );
                let _ = app.emit(
                    &format!("{}-complete", channel_prefix),
                    FfmpegResult {
                        success: false,
                        output_path: None,
                        error: Some(error_msg.clone()),
                    },
                );
                FfmpegResult {
                    success: false,
                    output_path: None,
                    error: Some(error_msg),
                }
            }
        }
        Err(e) => {
            let error_msg = format!("执行 ffmpeg 失败: {}", e);
            let _ = app.emit(
                &format!("{}-progress", channel_prefix),
                ProgressPayload {
                    status: "error".to_string(),
                    percent: None,
                    current_time: None,
                    total_duration,
                    index: None,
                    total: None,
                    file: None,
                },
            );
            let _ = app.emit(
                &format!("{}-log", channel_prefix),
                LogPayload {
                    time: now_string(),
                    message: error_msg.clone(),
                    log_type: "error".to_string(),
                },
            );
            let _ = app.emit(
                &format!("{}-complete", channel_prefix),
                FfmpegResult {
                    success: false,
                    output_path: None,
                    error: Some(error_msg.clone()),
                },
            );
            FfmpegResult {
                success: false,
                output_path: None,
                error: Some(error_msg),
            }
        }
    }
}
