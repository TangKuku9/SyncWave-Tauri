mod ffmpeg;
mod ncm;
mod playlist;

use ffmpeg::{FfmpegResult, LogPayload, ProgressPayload};
use playlist::{ImportResult, OperationResult, Playlist, PlaylistIndexEntry};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::process::Command as StdCommand;
use tauri::{AppHandle, Emitter};

// ============ File Selection Commands ============

#[tauri::command]
async fn select_files(app: AppHandle) -> Result<Vec<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let files = app
        .dialog()
        .file()
        .add_filter("NCM文件", &["ncm"])
        .add_filter("所有文件", &["*"])
        .set_title("选择NCM文件")
        .blocking_pick_files()
        .map(|f| f.into_iter().map(|p| p.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();
    Ok(files)
}

#[tauri::command]
async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let folder = app
        .dialog()
        .file()
        .set_title("选择输出目录")
        .blocking_pick_folder()
        .map(|p| p.to_string());
    Ok(folder)
}

#[tauri::command]
async fn select_video_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app
        .dialog()
        .file()
        .add_filter("视频文件", &["mp4", "mkv", "avi", "mov", "flv", "wmv", "webm", "ts"])
        .add_filter("所有文件", &["*"])
        .set_title("选择视频文件")
        .blocking_pick_file()
        .map(|p| p.to_string());
    Ok(file)
}

#[tauri::command]
async fn select_audio_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app
        .dialog()
        .file()
        .add_filter("音频文件", &["mp3", "m4a", "aac", "flac", "wav", "ogg", "wma", "opus"])
        .add_filter("所有文件", &["*"])
        .set_title("选择音频文件")
        .blocking_pick_file()
        .map(|p| p.to_string());
    Ok(file)
}

#[tauri::command]
async fn select_media_file(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app
        .dialog()
        .file()
        .add_filter("媒体文件", &["mp4", "mkv", "avi", "mov", "flv", "wmv", "webm", "ts", "mp3", "m4a", "aac", "flac", "wav", "ogg"])
        .add_filter("所有文件", &["*"])
        .set_title("选择媒体文件")
        .blocking_pick_file()
        .map(|p| p.to_string());
    Ok(file)
}

// ============ NCM Conversion Commands ============

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NcmConvertProgress {
    index: usize,
    total: usize,
    file: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NcmConvertLog {
    time: String,
    message: String,
    #[serde(rename = "type")]
    log_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct NcmConvertResult {
    file: String,
    success: bool,
    error: Option<String>,
}

#[tauri::command]
async fn convert_ncm(
    app: AppHandle,
    files: Vec<String>,
    output_dir: Option<String>,
    convert_type: Option<String>,
) -> Result<Vec<NcmConvertResult>, String> {
    let target_ext = convert_type.unwrap_or_else(|| "mp3".to_string());
    let mut results = Vec::new();

    let unlock_music_path = ffmpeg::get_unlock_music_path(&app);
    let ffmpeg_path = ffmpeg::get_ffmpeg_path(&app);
    let ffprobe_path = ffmpeg::get_ffprobe_path(&app);

    for (i, file) in files.iter().enumerate() {
        let file_name = Path::new(file)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let _ = app.emit("convert-progress", NcmConvertProgress {
            index: i, total: files.len(), file: file_name.clone(), status: "converting".to_string(),
        });

        let file_path = Path::new(file);
        let base_name = file_path.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let target_dir = output_dir.clone()
            .unwrap_or_else(|| file_path.parent().unwrap_or(Path::new(".")).to_string_lossy().to_string());

        // 确保输出目录存在
        let _ = fs::create_dir_all(&target_dir);

        if target_ext == "lyrics" {
            // === 歌词提取模式 ===
            let mut lyric_content: Option<String> = None;

            // 方法1: unlock-music 解密 → ffprobe 提取歌词
            if unlock_music_path.exists() && ffprobe_path.exists() {
                let temp_dir = Path::new(&target_dir).join("_um_temp");
                let _ = fs::create_dir_all(&temp_dir);

                let um_args = vec![
                    "-o".to_string(), temp_dir.to_string_lossy().to_string(),
                    "--overwrite".to_string(),
                    file.clone(),
                ];
                let um_result = StdCommand::new(&unlock_music_path)
                    .args(&um_args)
                    .output();

                if let Ok(output) = um_result {
                    if output.status.success() {
                        // 查找 unlock-music 输出的音频文件
                        let um_mp3 = temp_dir.join(format!("{}.mp3", base_name));
                        let um_flac = temp_dir.join(format!("{}.flac", base_name));
                        let um_file = if um_flac.exists() { Some(um_flac) } else if um_mp3.exists() { Some(um_mp3) } else { None };

                        if let Some(um_audio) = um_file {
                            // 用 ffprobe 从音频中提取歌词
                            if let Ok(ffout) = StdCommand::new(&ffprobe_path)
                                .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_entries", "format_tags", &um_audio.to_string_lossy()])
                                .output()
                            {
                                if let Ok(info) = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&ffout.stdout)) {
                                    if let Some(tags) = info.get("format").and_then(|f| f.get("tags")) {
                                        for key in &["lyrics", "LYRICS", "unsyncedlyrics", "USLT", "description"] {
                                            if let Some(val) = tags.get(*key).and_then(|v| v.as_str()) {
                                                if !val.trim().is_empty() {
                                                    lyric_content = Some(val.trim().to_string());
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // 清理临时目录
                let _ = fs::remove_dir_all(&temp_dir);
            }

            // 方法2: 内置解析器提取歌词（备选）
            if lyric_content.is_none() {
                if let Ok(info) = ncm::extract_ncm_info(file_path) {
                    if let Some(lyric) = info.metadata.lyric {
                        if !lyric.is_empty() {
                            lyric_content = Some(lyric);
                        }
                    }
                }
            }

            // 方法3: 外部 .lrc 文件（最终备选）
            if lyric_content.is_none() {
                let dir = file_path.parent().unwrap_or(Path::new("."));
                for lrc_name in &[format!("{}.lrc", base_name), format!("{}.LRC", base_name)] {
                    let lrc_path = dir.join(lrc_name);
                    if lrc_path.exists() {
                        if let Ok(content) = fs::read_to_string(&lrc_path) {
                            lyric_content = Some(content);
                            break;
                        }
                    }
                }
            }

            if let Some(lyric) = lyric_content {
                let lrc_path = Path::new(&target_dir).join(format!("{}.lrc", base_name));
                let _ = fs::write(&lrc_path, &lyric);
                let _ = app.emit("convert-log", NcmConvertLog {
                    time: chrono::Local::now().format("%H:%M:%S").to_string(),
                    message: format!("{} 歌词提取成功 → {}", file_name, lrc_path.file_name().unwrap_or_default().to_string_lossy()),
                    log_type: "success".to_string(),
                });
                results.push(NcmConvertResult { file: file_name.clone(), success: true, error: None });
            } else {
                let _ = app.emit("convert-log", NcmConvertLog {
                    time: chrono::Local::now().format("%H:%M:%S").to_string(),
                    message: format!("{} 未找到歌词信息", file_name),
                    log_type: "error".to_string(),
                });
                results.push(NcmConvertResult { file: file_name.clone(), success: false, error: Some("无歌词".to_string()) });
            }
        } else {
            // === 音频转换模式 (mp3/flac) ===
            if !unlock_music_path.exists() {
                let _ = app.emit("convert-log", NcmConvertLog {
                    time: chrono::Local::now().format("%H:%M:%S").to_string(),
                    message: format!("{} 未找到 unlock-music.exe", file_name),
                    log_type: "error".to_string(),
                });
                results.push(NcmConvertResult { file: file_name.clone(), success: false, error: Some("未找到 unlock-music.exe".to_string()) });
                continue;
            }

            // 用 unlock-music 解密（保留元数据、歌词、封面）
            let um_args = vec![
                "-o".to_string(), target_dir.clone(),
                "--overwrite".to_string(),
                file.clone(),
            ];
            let um_result = StdCommand::new(&unlock_music_path)
                .args(&um_args)
                .output();

            match um_result {
                Ok(output) if output.status.success() => {
                    // unlock-music 在 target_dir 生成原始格式文件
                    let um_mp3 = Path::new(&target_dir).join(format!("{}.mp3", base_name));
                    let um_flac = Path::new(&target_dir).join(format!("{}.flac", base_name));
                    let um_file = if um_flac.exists() { Some(um_flac) } else if um_mp3.exists() { Some(um_mp3) } else { None };

                    if let Some(um_audio) = um_file {
                        let um_ext = um_audio.extension().unwrap_or_default().to_string_lossy().to_string();
                        let output_path = Path::new(&target_dir).join(format!("{}.{}", base_name, target_ext));

                        if um_ext != target_ext && ffmpeg_path.exists() {
                            // 格式不同，用 ffmpeg 转码（-map_metadata 0 保留所有元数据）
                            let codec_args: Vec<String> = if target_ext == "flac" {
                                vec!["-c:a".into(), "flac".into(), "-map_metadata".into(), "0".into()]
                            } else {
                                vec!["-c:a".into(), "libmp3lame".into(), "-b:a".into(), "320k".into(), "-map_metadata".into(), "0".into()]
                            };
                            let args: Vec<String> = vec!["-i".into(), um_audio.to_string_lossy().to_string()]
                                .into_iter().chain(codec_args).chain(vec!["-y".into(), output_path.to_string_lossy().to_string()]).collect();

                            let result = StdCommand::new(&ffmpeg_path).args(&args).output();
                            // 删除 unlock-music 中间文件
                            let _ = fs::remove_file(&um_audio);

                            match result {
                                Ok(ffout) if ffout.status.success() => {
                                    // 尝试嵌入歌词：查找同目录 .lrc 文件
                                    let dir = file_path.parent().unwrap_or(Path::new("."));
                                    for lrc_name in &[format!("{}.lrc", base_name), format!("{}.LRC", base_name)] {
                                        let lrc_path = dir.join(lrc_name);
                                        if lrc_path.exists() {
                                            let _ = embed_lyrics(&app, &output_path, &lrc_path);
                                            break;
                                        }
                                    }
                                    let _ = app.emit("convert-log", NcmConvertLog {
                                        time: chrono::Local::now().format("%H:%M:%S").to_string(),
                                        message: format!("{} 转换成功 → {}", file_name, output_path.file_name().unwrap_or_default().to_string_lossy()),
                                        log_type: "success".to_string(),
                                    });
                                    results.push(NcmConvertResult { file: file_name.clone(), success: true, error: None });
                                }
                                Ok(ffout) => {
                                    let stderr = String::from_utf8_lossy(&ffout.stderr);
                                    let msg = if stderr.len() > 100 { &stderr[..100] } else { &stderr };
                                    let _ = app.emit("convert-log", NcmConvertLog {
                                        time: chrono::Local::now().format("%H:%M:%S").to_string(),
                                        message: format!("{} ffmpeg失败: {}", file_name, msg),
                                        log_type: "error".to_string(),
                                    });
                                    results.push(NcmConvertResult { file: file_name.clone(), success: false, error: Some("ffmpeg failed".to_string()) });
                                }
                                Err(e) => {
                                    results.push(NcmConvertResult { file: file_name.clone(), success: false, error: Some(e.to_string()) });
                                }
                            }
                        } else {
                            // 格式相同或无 ffmpeg，直接使用 unlock-music 输出
                            if um_ext != target_ext {
                                let _ = fs::rename(&um_audio, &output_path);
                            }
                            // 尝试嵌入歌词
                            let final_path = if um_ext != target_ext { &output_path } else { &um_audio };
                            let dir = file_path.parent().unwrap_or(Path::new("."));
                            for lrc_name in &[format!("{}.lrc", base_name), format!("{}.LRC", base_name)] {
                                let lrc_path = dir.join(lrc_name);
                                if lrc_path.exists() {
                                    let _ = embed_lyrics(&app, final_path, &lrc_path);
                                    break;
                                }
                            }
                            let _ = app.emit("convert-log", NcmConvertLog {
                                time: chrono::Local::now().format("%H:%M:%S").to_string(),
                                message: format!("{} 转换成功", file_name),
                                log_type: "success".to_string(),
                            });
                            results.push(NcmConvertResult { file: file_name.clone(), success: true, error: None });
                        }
                    } else {
                        let _ = app.emit("convert-log", NcmConvertLog {
                            time: chrono::Local::now().format("%H:%M:%S").to_string(),
                            message: format!("{} unlock-music 未生成输出文件", file_name),
                            log_type: "error".to_string(),
                        });
                        results.push(NcmConvertResult { file: file_name.clone(), success: false, error: Some("unlock-music 未生成输出文件".to_string()) });
                    }
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let msg = if stderr.len() > 200 { &stderr[..200] } else { &stderr };
                    let _ = app.emit("convert-log", NcmConvertLog {
                        time: chrono::Local::now().format("%H:%M:%S").to_string(),
                        message: format!("{} unlock-music 失败: {}", file_name, msg),
                        log_type: "error".to_string(),
                    });
                    results.push(NcmConvertResult { file: file_name.clone(), success: false, error: Some(format!("unlock-music failed: {}", msg)) });
                }
                Err(e) => {
                    let _ = app.emit("convert-log", NcmConvertLog {
                        time: chrono::Local::now().format("%H:%M:%S").to_string(),
                        message: format!("{} 启动 unlock-music 失败: {}", file_name, e),
                        log_type: "error".to_string(),
                    });
                    results.push(NcmConvertResult { file: file_name.clone(), success: false, error: Some(e.to_string()) });
                }
            }
        }

        let status = if results.last().map_or(false, |r| r.success) { "success" } else { "error" };
        let _ = app.emit("convert-progress", NcmConvertProgress {
            index: i, total: files.len(), file: file_name.clone(), status: status.to_string(),
        });
    }

    let _ = app.emit("convert-complete", &results);
    Ok(results)
}

/// 将 .lrc 歌词嵌入音频文件（MP3 ID3 USLT / FLAC Vorbis LYRICS）
fn embed_lyrics(app: &AppHandle, audio_path: &Path, lrc_path: &Path) -> bool {
    let ffmpeg_path = ffmpeg::get_ffmpeg_path(app);
    if !ffmpeg_path.exists() {
        return false;
    }
    let lrc_content = match fs::read_to_string(lrc_path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let ext = audio_path.extension().unwrap_or_default().to_string_lossy().to_lowercase();
    let temp_output = audio_path.with_extension(format!("tmp.{}", ext));

    let metadata_key = if ext == "flac" { "LYRICS" } else { "lyrics" };
    let args: Vec<String> = vec![
        "-i".into(), audio_path.to_string_lossy().to_string(),
        "-c".into(), "copy".into(),
        "-metadata".into(), format!("{}={}", metadata_key, lrc_content),
    ].into_iter().chain(if ext == "mp3" {
        vec!["-id3v2_version".into(), "3".into(), "-write_id3v2".into(), "1".into()]
    } else {
        vec![]
    }).chain(vec!["-y".into(), temp_output.to_string_lossy().to_string()]).collect();

    let result = StdCommand::new(&ffmpeg_path).args(&args).output();
    match result {
        Ok(output) if output.status.success() => {
            let _ = fs::remove_file(audio_path);
            let _ = fs::rename(&temp_output, audio_path);
            true
        }
        _ => {
            let _ = fs::remove_file(&temp_output);
            false
        }
    }
}

// ============ Music Player Commands ============

#[tauri::command]
async fn select_music_files(app: AppHandle) -> Result<Vec<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let files = app.dialog().file()
        .add_filter("音频文件", &["mp3", "flac", "wav", "ogg", "m4a", "aac", "wma"])
        .add_filter("所有文件", &["*"])
        .set_title("选择音乐文件")
        .blocking_pick_files()
        .map(|f| f.into_iter().map(|p| p.to_string()).collect::<Vec<_>>())
        .unwrap_or_default();
    Ok(files)
}

#[tauri::command]
async fn scan_directory(dir_path: String) -> Result<Vec<String>, String> {
    let path = Path::new(&dir_path);
    if !path.exists() { return Ok(Vec::new()); }
    let audio_exts = [".mp3", ".flac", ".wav", ".ogg", ".m4a", ".aac", ".wma"];
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let ext = entry.path().extension().unwrap_or_default().to_string_lossy().to_lowercase();
            if audio_exts.contains(&format!(".{}", ext).as_str()) {
                files.push(entry.path().to_string_lossy().to_string());
            }
        }
    }
    Ok(files)
}

#[tauri::command]
async fn get_music_url(file_path: String) -> Result<String, String> {
    Ok(format!("file:///{}", file_path.replace('\\', "/")))
}

#[tauri::command]
async fn read_audio_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))
}

#[derive(Debug, Serialize, Deserialize)]
struct AudioMetadata {
    path: String, name: String, size: u64, exists: bool, lyrics: Option<String>, error: Option<String>,
}

#[tauri::command]
async fn get_audio_metadata(app: AppHandle, file_path: String) -> Result<AudioMetadata, String> {
    let path = Path::new(&file_path);
    let base_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
    let dir = path.parent().unwrap_or(Path::new("."));
    match fs::metadata(path) {
        Ok(stats) => {
            let mut lyrics = None;
            let ffprobe_path = ffmpeg::get_ffprobe_path(&app);
            if ffprobe_path.exists() {
                if let Ok(output) = StdCommand::new(&ffprobe_path)
                    .args(["-v", "quiet", "-print_format", "json", "-show_format", "-show_entries", "format_tags", &file_path])
                    .output()
                {
                    if let Ok(info) = serde_json::from_str::<serde_json::Value>(&String::from_utf8_lossy(&output.stdout)) {
                        if let Some(tags) = info.get("format").and_then(|f| f.get("tags")) {
                            for key in &["lyrics", "LYRICS", "unsyncedlyrics", "USLT", "description"] {
                                if let Some(val) = tags.get(*key).and_then(|v| v.as_str()) {
                                    if !val.trim().is_empty() { lyrics = Some(val.trim().to_string()); break; }
                                }
                            }
                        }
                    }
                }
            }
            if lyrics.is_none() {
                for lrc in &[dir.join(format!("{}.lrc", base_name)), dir.join(format!("{}.LRC", base_name))] {
                    if lrc.exists() {
                        if let Ok(content) = fs::read_to_string(lrc) { lyrics = Some(content); break; }
                    }
                }
            }
            Ok(AudioMetadata { path: file_path, name: base_name, size: stats.len(), exists: true, lyrics, error: None })
        }
        Err(e) => Ok(AudioMetadata { path: file_path, name: base_name, size: 0, exists: false, lyrics: None, error: Some(e.to_string()) }),
    }
}

// ============ FFmpeg Tool Commands ============

fn get_output_path(input_path: &str, suffix: &str, ext: &str, output_dir: &Option<String>) -> String {
    let input = Path::new(input_path);
    let base_name = input.file_stem().unwrap_or_default().to_string_lossy();
    let dir = output_dir.as_deref().unwrap_or_else(|| input.parent().unwrap_or(Path::new(".")).to_str().unwrap_or("."));
    Path::new(dir).join(format!("{}{}.{}", base_name, suffix, ext)).to_string_lossy().to_string()
}

#[tauri::command]
async fn merge_av(app: AppHandle, video_path: String, audio_path: String, output_dir: Option<String>) -> Result<FfmpegResult, String> {
    let output_path = get_output_path(&video_path, "_merged", "mp4", &output_dir);
    let _ = app.emit("merge-log", LogPayload {
        time: chrono::Local::now().format("%H:%M:%S").to_string(),
        message: format!("开始合并: {} + {}",
            Path::new(&video_path).file_name().unwrap_or_default().to_string_lossy(),
            Path::new(&audio_path).file_name().unwrap_or_default().to_string_lossy()),
        log_type: "info".to_string(),
    });
    let args = vec![
        "-i".into(), video_path.clone(), "-i".into(), audio_path,
        "-c:v".into(), "copy".into(), "-c:a".into(), "aac".into(),
        "-map".into(), "0:v:0".into(), "-map".into(), "1:a:0".into(),
        "-y".into(), output_path.clone(),
    ];
    Ok(ffmpeg::run_ffmpeg(&app, &args, "merge", &output_path, Some(&video_path)))
}

#[tauri::command]
async fn format_convert(app: AppHandle, input_path: String, output_format: String, output_dir: Option<String>) -> Result<FfmpegResult, String> {
    let output_path = get_output_path(&input_path, "", &output_format, &output_dir);
    let _ = app.emit("convert-format-log", LogPayload {
        time: chrono::Local::now().format("%H:%M:%S").to_string(),
        message: format!("开始转换: {} → {}", Path::new(&input_path).file_name().unwrap_or_default().to_string_lossy(), output_format),
        log_type: "info".to_string(),
    });
    let args = match output_format.as_str() {
        "mp4"|"mkv"|"avi"|"mov"|"webm" => vec!["-i".into(), input_path.clone(), "-c:v".into(), "libx264".into(), "-preset".into(), "fast".into(), "-crf".into(), "23".into(), "-c:a".into(), "aac".into(), "-b:a".into(), "192k".into(), "-y".into(), output_path.clone()],
        "mp3" => vec!["-i".into(), input_path.clone(), "-vn".into(), "-c:a".into(), "libmp3lame".into(), "-b:a".into(), "320k".into(), "-y".into(), output_path.clone()],
        "flac" => vec!["-i".into(), input_path.clone(), "-vn".into(), "-c:a".into(), "flac".into(), "-y".into(), output_path.clone()],
        "wav" => vec!["-i".into(), input_path.clone(), "-vn".into(), "-c:a".into(), "pcm_s16le".into(), "-y".into(), output_path.clone()],
        "m4a"|"aac" => vec!["-i".into(), input_path.clone(), "-vn".into(), "-c:a".into(), "aac".into(), "-b:a".into(), "256k".into(), "-y".into(), output_path.clone()],
        "ogg" => vec!["-i".into(), input_path.clone(), "-vn".into(), "-c:a".into(), "libvorbis".into(), "-b:a".into(), "192k".into(), "-y".into(), output_path.clone()],
        "opus" => vec!["-i".into(), input_path.clone(), "-vn".into(), "-c:a".into(), "libopus".into(), "-b:a".into(), "128k".into(), "-y".into(), output_path.clone()],
        _ => vec!["-i".into(), input_path.clone(), "-y".into(), output_path.clone()],
    };
    Ok(ffmpeg::run_ffmpeg(&app, &args, "convert-format", &output_path, Some(&input_path)))
}

#[tauri::command]
async fn extract_audio(app: AppHandle, input_path: String, audio_format: String, output_dir: Option<String>) -> Result<FfmpegResult, String> {
    let output_path = get_output_path(&input_path, "", &audio_format, &output_dir);
    let _ = app.emit("extract-audio-log", LogPayload {
        time: chrono::Local::now().format("%H:%M:%S").to_string(),
        message: format!("开始提取音频: {} → {}", Path::new(&input_path).file_name().unwrap_or_default().to_string_lossy(), audio_format),
        log_type: "info".to_string(),
    });
    let codec: Vec<String> = match audio_format.as_str() {
        "mp3" => vec!["-vn".to_string(), "-c:a".to_string(), "libmp3lame".to_string(), "-b:a".to_string(), "320k".to_string()],
        "flac" => vec!["-vn".to_string(), "-c:a".to_string(), "flac".to_string()],
        "wav" => vec!["-vn".to_string(), "-c:a".to_string(), "pcm_s16le".to_string()],
        "m4a"|"aac" => vec!["-vn".to_string(), "-c:a".to_string(), "aac".to_string(), "-b:a".to_string(), "256k".to_string()],
        "ogg" => vec!["-vn".to_string(), "-c:a".to_string(), "libvorbis".to_string(), "-b:a".to_string(), "192k".to_string()],
        _ => vec!["-vn".to_string(), "-c:a".to_string(), "copy".to_string()],
    };
    let args: Vec<String> = vec!["-i".into(), input_path.clone()].into_iter().chain(codec).chain(vec!["-y".into(), output_path.clone()]).collect();
    Ok(ffmpeg::run_ffmpeg(&app, &args, "extract-audio", &output_path, Some(&input_path)))
}

#[tauri::command]
async fn clip_video(app: AppHandle, input_path: String, start_time: String, end_time: String, output_dir: Option<String>) -> Result<FfmpegResult, String> {
    let ext = Path::new(&input_path).extension().unwrap_or_default().to_string_lossy().to_string();
    let output_path = get_output_path(&input_path, "_clip", &ext, &output_dir);
    let _ = app.emit("clip-video-log", LogPayload {
        time: chrono::Local::now().format("%H:%M:%S").to_string(),
        message: format!("开始截取: {} → {}", start_time, end_time), log_type: "info".to_string(),
    });
    let args = vec!["-ss".into(), start_time, "-to".into(), end_time, "-i".into(), input_path.clone(), "-c".into(), "copy".into(), "-y".into(), output_path.clone()];
    Ok(ffmpeg::run_ffmpeg(&app, &args, "clip-video", &output_path, Some(&input_path)))
}

#[tauri::command]
async fn compress_video(app: AppHandle, input_path: String, crf: String, scale: Option<String>, output_dir: Option<String>) -> Result<FfmpegResult, String> {
    let output_path = get_output_path(&input_path, "_compressed", "mp4", &output_dir);
    let _ = app.emit("compress-video-log", LogPayload {
        time: chrono::Local::now().format("%H:%M:%S").to_string(),
        message: format!("开始压缩: CRF={}, 分辨率={}", crf, scale.as_deref().unwrap_or("原分辨率")),
        log_type: "info".to_string(),
    });
    let mut args = vec!["-i".into(), input_path.clone(), "-c:v".into(), "libx264".into(), "-preset".into(), "medium".into(), "-crf".into(), crf];
    if let Some(ref s) = scale { if s != "original" { args.extend(vec!["-vf".into(), format!("scale={}", s)]); } }
    args.extend(vec!["-c:a".into(), "aac".into(), "-b:a".into(), "128k".into(), "-y".into(), output_path.clone()]);
    Ok(ffmpeg::run_ffmpeg(&app, &args, "compress-video", &output_path, Some(&input_path)))
}

#[tauri::command]
async fn video_to_gif(app: AppHandle, input_path: String, fps: Option<u32>, width: Option<u32>, aspect_ratio: Option<String>, start_time: Option<String>, end_time: Option<String>, output_dir: Option<String>) -> Result<FfmpegResult, String> {
    let output_path = get_output_path(&input_path, "", "gif", &output_dir);
    let fps_val = fps.unwrap_or(10);
    let width_val = width.unwrap_or(320);
    let scale_filter = if let Some(ref ar) = aspect_ratio {
        if ar != "original" {
            let parts: Vec<&str> = ar.split(':').collect();
            if parts.len() == 2 {
                let w: u32 = parts[0].parse().unwrap_or(1);
                let h: u32 = parts[1].parse().unwrap_or(1);
                if w == h { format!("crop='min(iw\\,ih)':'min(iw\\,ih)',scale={}:{}:flags=lanczos", width_val, width_val) }
                else if w > h { let th = (width_val as f64 * h as f64 / w as f64).round() as u32; format!("crop='iw':'iw*{}/{}',scale={}:{}:flags=lanczos", h, w, width_val, th) }
                else { let th = (width_val as f64 * h as f64 / w as f64).round() as u32; format!("crop='ih*{}/{}':'ih',scale={}:{}:flags=lanczos", w, h, width_val, th) }
            } else { format!("scale={}:-1:flags=lanczos", width_val) }
        } else { format!("scale={}:-1:flags=lanczos", width_val) }
    } else { format!("scale={}:-1:flags=lanczos", width_val) };

    let _ = app.emit("video-to-gif-log", LogPayload {
        time: chrono::Local::now().format("%H:%M:%S").to_string(),
        message: format!("开始转换GIF: 帧率={}fps, 宽度={}px", fps_val, width_val), log_type: "info".to_string(),
    });

    let ffmpeg_path = ffmpeg::get_ffmpeg_path(&app);
    if !ffmpeg_path.exists() { return Ok(FfmpegResult { success: false, output_path: None, error: Some("ffmpeg not found".into()) }); }

    let mut time_args: Vec<String> = Vec::new();
    if let Some(ref st) = start_time { time_args.extend(vec!["-ss".into(), st.clone()]); }
    if let Some(ref et) = end_time { time_args.extend(vec!["-to".into(), et.clone()]); }

    let palette_path = get_output_path(&input_path, "_palette", "png", &output_dir);
    let palette_filter = format!("fps={},{},palettegen=stats_mode=diff", fps_val, scale_filter);
    let palette_args: Vec<String> = time_args.iter().cloned().chain(vec!["-i".into(), input_path.clone(), "-vf".into(), palette_filter, "-y".into(), palette_path.clone()]).collect();

    if StdCommand::new(&ffmpeg_path).args(&palette_args).output().is_err() {
        return Ok(FfmpegResult { success: false, output_path: None, error: Some("调色板生成失败".into()) });
    }

    let gif_filter = format!("fps={},{}[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle", fps_val, scale_filter);
    let gif_args: Vec<String> = time_args.iter().cloned().chain(vec!["-i".into(), input_path.clone(), "-i".into(), palette_path.clone(), "-lavfi".into(), gif_filter, "-y".into(), output_path.clone()]).collect();
    let _ = fs::remove_file(&palette_path);

    match StdCommand::new(&ffmpeg_path).args(&gif_args).output() {
        Ok(output) if output.status.success() && Path::new(&output_path).exists() => {
            let size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);
            let _ = app.emit("video-to-gif-progress", ProgressPayload { status: "done".into(), percent: Some(100.0), current_time: None, total_duration: None, index: None, total: None, file: None });
            let _ = app.emit("video-to-gif-log", LogPayload { time: chrono::Local::now().format("%H:%M:%S").to_string(), message: format!("完成: {:.1} MB", size as f64 / 1048576.0), log_type: "success".into() });
            Ok(FfmpegResult { success: true, output_path: Some(output_path), error: None })
        }
        _ => Ok(FfmpegResult { success: false, output_path: None, error: Some("GIF生成失败".into()) }),
    }
}

// ============ External Player Commands ============

#[tauri::command]
async fn play_with_ffplay(app: AppHandle, file_path: String) -> Result<(), String> {
    let ffplay_path = ffmpeg::get_ffplay_path(&app);
    if !ffplay_path.exists() {
        return Err("未找到 ffplay.exe".to_string());
    }
    // 使用 ffplay 无界面模式播放音频，播放完毕后自动退出
    std::thread::spawn(move || {
        let _ = std::process::Command::new(&ffplay_path)
            .args(["-nodisp", "-autoexit", "-loglevel", "quiet", &file_path])
            .output();
    });
    Ok(())
}

// ============ Playlist Commands ============

#[tauri::command]
fn load_playlists_index_cmd(app: AppHandle) -> Vec<PlaylistIndexEntry> { playlist::load_playlists_index(&app) }

#[tauri::command]
fn load_playlist_cmd(app: AppHandle, playlist_id: String) -> Playlist { playlist::load_playlist(&app, &playlist_id) }

#[tauri::command]
fn save_playlist_cmd(app: AppHandle, playlist: Playlist) -> OperationResult { playlist::save_playlist(&app, &playlist) }

#[tauri::command]
fn create_playlist_cmd(app: AppHandle, name: String) -> (Option<String>, Option<String>) { playlist::create_playlist(&app, &name) }

#[tauri::command]
fn delete_playlist_cmd(app: AppHandle, playlist_id: String) -> OperationResult { playlist::delete_playlist(&app, &playlist_id) }

#[tauri::command]
fn rename_playlist_cmd(app: AppHandle, playlist_id: String, new_name: String) -> OperationResult { playlist::rename_playlist(&app, &playlist_id, &new_name) }

#[tauri::command]
fn reorder_playlists_cmd(app: AppHandle, ordered_ids: Vec<String>) -> OperationResult { playlist::reorder_playlists(&app, &ordered_ids) }

#[tauri::command]
async fn import_playlist_cmd(app: AppHandle) -> Result<ImportResult, String> {
    use tauri_plugin_dialog::DialogExt;
    let file = app.dialog().file()
        .add_filter("歌单文件", &["json", "m3u", "m3u8"])
        .set_title("导入歌单")
        .blocking_pick_file().map(|p| p.to_string());
    match file {
        Some(path) => Ok(playlist::import_playlist(&app, &path)),
        None => Ok(ImportResult { id: None, name: None, count: 0, failed: 0, error: None }),
    }
}

#[tauri::command]
async fn export_playlist_json_cmd(app: AppHandle, playlist_id: String) -> Result<OperationResult, String> {
    use tauri_plugin_dialog::DialogExt;
    let pl = playlist::load_playlist(&app, &playlist_id);
    let name = format!("{}.json", if pl.name.is_empty() { "未命名歌单" } else { &pl.name });
    let save = app.dialog().file().set_title("导出歌单").add_filter("JSON", &["json"]).set_file_name(&name).blocking_save_file().map(|p| p.to_string());
    match save {
        Some(path) => Ok(playlist::export_playlist_json(&app, &playlist_id, &path)),
        None => Ok(OperationResult { success: false, error: Some("用户取消".into()), file_path: None }),
    }
}

#[tauri::command]
async fn export_playlist_m3u_cmd(app: AppHandle, playlist_id: String) -> Result<OperationResult, String> {
    use tauri_plugin_dialog::DialogExt;
    let pl = playlist::load_playlist(&app, &playlist_id);
    let name = format!("{}.m3u", if pl.name.is_empty() { "未命名歌单" } else { &pl.name });
    let save = app.dialog().file().set_title("导出歌单").add_filter("M3U", &["m3u"]).set_file_name(&name).blocking_save_file().map(|p| p.to_string());
    match save {
        Some(path) => Ok(playlist::export_playlist_m3u(&app, &playlist_id, &path)),
        None => Ok(OperationResult { success: false, error: Some("用户取消".into()), file_path: None }),
    }
}

// ============ App Entry ============

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            select_files, select_folder, select_video_file, select_audio_file, select_media_file,
            convert_ncm,
            select_music_files, scan_directory, get_music_url, get_audio_metadata, read_audio_file,
            merge_av, format_convert, extract_audio, clip_video, compress_video, video_to_gif,
            play_with_ffplay,
            load_playlists_index_cmd, load_playlist_cmd, save_playlist_cmd, create_playlist_cmd,
            delete_playlist_cmd, rename_playlist_cmd, reorder_playlists_cmd,
            import_playlist_cmd, export_playlist_json_cmd, export_playlist_m3u_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
