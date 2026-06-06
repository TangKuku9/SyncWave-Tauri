use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistIndexEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub is_default: bool,
    pub created: String,
    pub modified: String,
    #[serde(default)]
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlaylistIndexData {
    pub version: String,
    pub playlists: Vec<PlaylistIndexEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Song {
    #[serde(default)]
    pub id: String,
    pub path: String,
    pub name: String,
    #[serde(default)]
    pub duration: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub created: String,
    pub modified: String,
    #[serde(default)]
    pub songs: Vec<Song>,
}

fn get_playlists_dir(app: &AppHandle) -> PathBuf {
    let data_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let dir = data_dir.join("playlists");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    dir
}

fn get_index_path(app: &AppHandle) -> PathBuf {
    get_playlists_dir(app).join("playlistsIndex.json")
}

fn get_playlist_path(app: &AppHandle, id: &str) -> PathBuf {
    get_playlists_dir(app).join(format!("{}.json", id))
}

fn now_iso() -> String {
    chrono::Local::now().to_rfc3339()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportResult {
    pub id: Option<String>,
    pub name: Option<String>,
    pub count: usize,
    pub failed: usize,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OperationResult {
    pub success: bool,
    pub error: Option<String>,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
}

fn parse_m3u_file(file_path: &Path) -> Vec<Song> {
    let content = fs::read_to_string(file_path).unwrap_or_default();
    let lines: Vec<&str> = content.split('\n').collect();
    let mut songs = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_duration: f64 = 0.0;
    let m3u_dir = file_path.parent().unwrap_or(Path::new("."));

    for line in &lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "#EXTM3U" {
            continue;
        }

        if trimmed.starts_with("#EXTINF:") {
            let after_prefix = &trimmed[8..];
            if let Some(comma_pos) = after_prefix.find(',') {
                let dur_str = &after_prefix[..comma_pos];
                current_duration = dur_str.parse::<f64>().unwrap_or(0.0).round();
                current_name = Some(after_prefix[comma_pos + 1..].trim().to_string());
            }
        } else if !trimmed.starts_with('#') {
            let song_path = if Path::new(trimmed).is_absolute() {
                trimmed.to_string()
            } else {
                m3u_dir.join(trimmed).to_string_lossy().to_string()
            };

            let name = current_name
                .take()
                .unwrap_or_else(|| {
                    Path::new(&song_path)
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string()
                });

            songs.push(Song {
                id: format!("song-{}-{}", chrono::Local::now().timestamp_millis(), &uuid::Uuid::new_v4().to_string()[..8]),
                path: song_path,
                name,
                duration: current_duration,
            });
            current_duration = 0.0;
        }
    }
    songs
}

/// Load playlists index
pub fn load_playlists_index(app: &AppHandle) -> Vec<PlaylistIndexEntry> {
    let index_path = get_index_path(app);
    let dir = get_playlists_dir(app);

    if !index_path.exists() {
        // Create default playlist
        let now = now_iso();
        let default_playlist = Playlist {
            id: "default".to_string(),
            name: "默认歌单".to_string(),
            created: now.clone(),
            modified: now.clone(),
            songs: Vec::new(),
        };
        let _ = fs::write(
            dir.join("default.json"),
            serde_json::to_string_pretty(&default_playlist).unwrap(),
        );
        let index_data = PlaylistIndexData {
            version: "1.0".to_string(),
            playlists: vec![PlaylistIndexEntry {
                id: "default".to_string(),
                name: "默认歌单".to_string(),
                is_default: true,
                created: now.clone(),
                modified: now,
                count: 0,
            }],
        };
        let _ = fs::write(
            &index_path,
            serde_json::to_string_pretty(&index_data).unwrap(),
        );
        return index_data.playlists;
    }

    match fs::read_to_string(&index_path) {
        Ok(content) => match serde_json::from_str::<PlaylistIndexData>(&content) {
            Ok(data) => data.playlists,
            Err(_) => Vec::new(),
        },
        Err(_) => Vec::new(),
    }
}

/// Load a single playlist
pub fn load_playlist(app: &AppHandle, playlist_id: &str) -> Playlist {
    let path = get_playlist_path(app, playlist_id);
    if !path.exists() {
        return Playlist {
            id: playlist_id.to_string(),
            name: String::new(),
            created: String::new(),
            modified: String::new(),
            songs: Vec::new(),
        };
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or(Playlist {
            id: playlist_id.to_string(),
            name: String::new(),
            created: String::new(),
            modified: String::new(),
            songs: Vec::new(),
        }),
        Err(_) => Playlist {
            id: playlist_id.to_string(),
            name: String::new(),
            created: String::new(),
            modified: String::new(),
            songs: Vec::new(),
        },
    }
}

/// Save a playlist
pub fn save_playlist(app: &AppHandle, playlist: &Playlist) -> OperationResult {
    let dir = get_playlists_dir(app);
    let playlist_path = dir.join(format!("{}.json", playlist.id));
    let now = now_iso();

    let mut pl = playlist.clone();
    pl.modified = now.clone();

    match fs::write(&playlist_path, serde_json::to_string_pretty(&pl).unwrap()) {
        Ok(_) => {
            // Update index
            let index_path = get_index_path(app);
            if let Ok(content) = fs::read_to_string(&index_path) {
                if let Ok(mut index_data) = serde_json::from_str::<PlaylistIndexData>(&content) {
                    if let Some(entry) = index_data.playlists.iter_mut().find(|p| p.id == pl.id) {
                        entry.modified = now;
                        entry.count = pl.songs.len();
                        if !pl.name.is_empty() {
                            entry.name = pl.name.clone();
                        }
                    }
                    let _ = fs::write(
                        &index_path,
                        serde_json::to_string_pretty(&index_data).unwrap(),
                    );
                }
            }
            OperationResult {
                success: true,
                error: None,
                file_path: None,
            }
        }
        Err(e) => OperationResult {
            success: false,
            error: Some(e.to_string()),
            file_path: None,
        },
    }
}

/// Create a new playlist
pub fn create_playlist(app: &AppHandle, name: &str) -> (Option<String>, Option<String>) {
    let dir = get_playlists_dir(app);
    let id = format!("playlist-{}", chrono::Local::now().timestamp_millis());
    let now = now_iso();

    let new_playlist = Playlist {
        id: id.clone(),
        name: name.to_string(),
        created: now.clone(),
        modified: now.clone(),
        songs: Vec::new(),
    };

    let _ = fs::write(
        dir.join(format!("{}.json", id)),
        serde_json::to_string_pretty(&new_playlist).unwrap(),
    );

    // Update index
    let index_path = get_index_path(app);
    let mut index_data = if let Ok(content) = fs::read_to_string(&index_path) {
        serde_json::from_str::<PlaylistIndexData>(&content).unwrap_or(PlaylistIndexData {
            version: "1.0".to_string(),
            playlists: Vec::new(),
        })
    } else {
        PlaylistIndexData {
            version: "1.0".to_string(),
            playlists: Vec::new(),
        }
    };

    index_data.playlists.push(PlaylistIndexEntry {
        id: id.clone(),
        name: name.to_string(),
        is_default: false,
        created: now.clone(),
        modified: now,
        count: 0,
    });

    let _ = fs::write(
        &index_path,
        serde_json::to_string_pretty(&index_data).unwrap(),
    );

    (Some(id), Some(name.to_string()))
}

/// Delete a playlist
pub fn delete_playlist(app: &AppHandle, playlist_id: &str) -> OperationResult {
    let index_path = get_index_path(app);

    let content = match fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(_) => {
            return OperationResult {
                success: false,
                error: Some("索引文件不存在".to_string()),
                file_path: None,
            }
        }
    };

    let mut index_data: PlaylistIndexData = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(_) => {
            return OperationResult {
                success: false,
                error: Some("索引文件格式错误".to_string()),
                file_path: None,
            }
        }
    };

    let entry = index_data.playlists.iter().find(|p| p.id == playlist_id);
    if entry.is_none() {
        return OperationResult {
            success: false,
            error: Some("歌单不存在".to_string()),
            file_path: None,
        };
    }
    if entry.unwrap().is_default {
        return OperationResult {
            success: false,
            error: Some("不能删除默认歌单".to_string()),
            file_path: None,
        };
    }

    // Delete playlist file
    let playlist_path = get_playlist_path(app, playlist_id);
    let _ = fs::remove_file(&playlist_path);

    // Update index
    index_data.playlists.retain(|p| p.id != playlist_id);
    let _ = fs::write(
        &index_path,
        serde_json::to_string_pretty(&index_data).unwrap(),
    );

    OperationResult {
        success: true,
        error: None,
        file_path: None,
    }
}

/// Rename a playlist
pub fn rename_playlist(app: &AppHandle, playlist_id: &str, new_name: &str) -> OperationResult {
    let now = now_iso();

    // Update playlist file
    let playlist_path = get_playlist_path(app, playlist_id);
    if let Ok(content) = fs::read_to_string(&playlist_path) {
        if let Ok(mut playlist) = serde_json::from_str::<Playlist>(&content) {
            playlist.name = new_name.to_string();
            playlist.modified = now.clone();
            let _ = fs::write(
                &playlist_path,
                serde_json::to_string_pretty(&playlist).unwrap(),
            );
        }
    }

    // Update index
    let index_path = get_index_path(app);
    if let Ok(content) = fs::read_to_string(&index_path) {
        if let Ok(mut index_data) = serde_json::from_str::<PlaylistIndexData>(&content) {
            if let Some(entry) = index_data.playlists.iter_mut().find(|p| p.id == playlist_id) {
                entry.name = new_name.to_string();
                entry.modified = now;
            }
            let _ = fs::write(
                &index_path,
                serde_json::to_string_pretty(&index_data).unwrap(),
            );
        }
    }

    OperationResult {
        success: true,
        error: None,
        file_path: None,
    }
}

/// Reorder playlists
pub fn reorder_playlists(app: &AppHandle, ordered_ids: &[String]) -> OperationResult {
    let index_path = get_index_path(app);

    let content = match fs::read_to_string(&index_path) {
        Ok(c) => c,
        Err(_) => {
            return OperationResult {
                success: false,
                error: Some("索引文件不存在".to_string()),
                file_path: None,
            }
        }
    };

    let mut index_data: PlaylistIndexData = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(_) => {
            return OperationResult {
                success: false,
                error: Some("索引文件格式错误".to_string()),
                file_path: None,
            }
        }
    };

    let playlists_map: std::collections::HashMap<String, PlaylistIndexEntry> = index_data
        .playlists
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect();

    let mut reordered = Vec::new();
    for id in ordered_ids {
        if let Some(pl) = playlists_map.get(id) {
            reordered.push(pl.clone());
        }
    }
    // Add any missing ones
    for (_, pl) in &playlists_map {
        if !ordered_ids.contains(&pl.id) {
            reordered.push(pl.clone());
        }
    }

    index_data.playlists = reordered;
    let _ = fs::write(
        &index_path,
        serde_json::to_string_pretty(&index_data).unwrap(),
    );

    OperationResult {
        success: true,
        error: None,
        file_path: None,
    }
}

/// Import playlist from file
pub fn import_playlist(app: &AppHandle, file_path: &str) -> ImportResult {
    let path = Path::new(file_path);
    if !path.exists() {
        return ImportResult {
            id: None,
            name: None,
            count: 0,
            failed: 0,
            error: Some("文件不存在".to_string()),
        };
    }

    let ext = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    let mut playlist_name = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let songs = if ext == "json" {
        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(data) => {
                    if let Some(songs_val) = data.get("songs") {
                        if let Some(name) = data.get("name").and_then(|n| n.as_str()) {
                            playlist_name = name.to_string();
                        }
                        serde_json::from_value::<Vec<Song>>(songs_val.clone()).unwrap_or_default()
                    } else if data.is_array() {
                        serde_json::from_value::<Vec<Song>>(data).unwrap_or_default()
                    } else {
                        return ImportResult {
                            id: None,
                            name: None,
                            count: 0,
                            failed: 0,
                            error: Some("无效的歌单JSON格式".to_string()),
                        };
                    }
                }
                Err(e) => {
                    return ImportResult {
                        id: None,
                        name: None,
                        count: 0,
                        failed: 0,
                        error: Some(format!("JSON解析失败: {}", e)),
                    }
                }
            },
            Err(e) => {
                return ImportResult {
                    id: None,
                    name: None,
                    count: 0,
                    failed: 0,
                    error: Some(format!("读取文件失败: {}", e)),
                }
            }
        }
    } else if ext == "m3u" || ext == "m3u8" {
        parse_m3u_file(path)
    } else {
        return ImportResult {
            id: None,
            name: None,
            count: 0,
            failed: 0,
            error: Some("不支持的文件格式".to_string()),
        };
    };

    // Validate songs
    let mut failed = 0usize;
    let mut valid_songs = Vec::new();
    for mut song in songs {
        if Path::new(&song.path).exists() {
            if song.id.is_empty() {
                song.id = format!("song-{}-{}", chrono::Local::now().timestamp_millis(), &uuid::Uuid::new_v4().to_string()[..8]);
            }
            valid_songs.push(song);
        } else {
            failed += 1;
        }
    }

    let count = valid_songs.len();

    // Create playlist
    let dir = get_playlists_dir(app);
    let id = format!("playlist-{}", chrono::Local::now().timestamp_millis());
    let now = now_iso();

    let new_playlist = Playlist {
        id: id.clone(),
        name: playlist_name.clone(),
        created: now.clone(),
        modified: now.clone(),
        songs: valid_songs,
    };

    let _ = fs::write(
        dir.join(format!("{}.json", id)),
        serde_json::to_string_pretty(&new_playlist).unwrap(),
    );

    // Update index
    let index_path = get_index_path(app);
    let mut index_data = if let Ok(content) = fs::read_to_string(&index_path) {
        serde_json::from_str::<PlaylistIndexData>(&content).unwrap_or(PlaylistIndexData {
            version: "1.0".to_string(),
            playlists: Vec::new(),
        })
    } else {
        PlaylistIndexData {
            version: "1.0".to_string(),
            playlists: Vec::new(),
        }
    };

    index_data.playlists.push(PlaylistIndexEntry {
        id: id.clone(),
        name: playlist_name.clone(),
        is_default: false,
        created: now.clone(),
        modified: now,
        count,
    });

    let _ = fs::write(
        &index_path,
        serde_json::to_string_pretty(&index_data).unwrap(),
    );

    ImportResult {
        id: Some(id),
        name: Some(playlist_name),
        count,
        failed,
        error: None,
    }
}

/// Export playlist as JSON
pub fn export_playlist_json(app: &AppHandle, playlist_id: &str, save_path: &str) -> OperationResult {
    let playlist_path = get_playlist_path(app, playlist_id);
    if !playlist_path.exists() {
        return OperationResult {
            success: false,
            error: Some("歌单不存在".to_string()),
            file_path: None,
        };
    }

    match fs::read_to_string(&playlist_path) {
        Ok(content) => match fs::write(save_path, &content) {
            Ok(_) => OperationResult {
                success: true,
                error: None,
                file_path: Some(save_path.to_string()),
            },
            Err(e) => OperationResult {
                success: false,
                error: Some(e.to_string()),
                file_path: None,
            },
        },
        Err(e) => OperationResult {
            success: false,
            error: Some(e.to_string()),
            file_path: None,
        },
    }
}

/// Export playlist as M3U
pub fn export_playlist_m3u(app: &AppHandle, playlist_id: &str, save_path: &str) -> OperationResult {
    let playlist = load_playlist(app, playlist_id);

    let mut m3u_content = String::from("#EXTM3U\n");
    for song in &playlist.songs {
        m3u_content.push_str(&format!("#EXTINF:{},{}\n", song.duration as i64, song.name));
        m3u_content.push_str(&format!("{}\n", song.path));
    }

    match fs::write(save_path, m3u_content) {
        Ok(_) => OperationResult {
            success: true,
            error: None,
            file_path: Some(save_path.to_string()),
        },
        Err(e) => OperationResult {
            success: false,
            error: Some(e.to_string()),
            file_path: None,
        },
    }
}
