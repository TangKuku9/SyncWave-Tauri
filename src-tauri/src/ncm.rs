use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyInit};
use aes::Aes128;
use ecb::Decryptor as EcbDecryptor;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

type Aes128EcbDec = EcbDecryptor<Aes128>;

const NCM_CORE_KEY: &[u8] = b"hzHRAmso5Kn2AW"; // 687A4852416D736F356B496E324157
const NCM_META_KEY: &[u8] = b"#14ljk_!\\]&0U<'("; // 2331346C6A6B5F215C5D2630553C2728
const MAGIC: &[u8] = b"CTENFDAM";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NcmMetadata {
    #[serde(rename = "musicName")]
    pub music_name: Option<String>,
    pub artist: Option<Vec<String>>,
    pub album: Option<String>,
    pub format: Option<String>,
    pub lyric: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NcmInfo {
    pub metadata: NcmMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NcmResult {
    pub music_data: Vec<u8>,
    pub format: String,
    pub metadata: NcmMetadata,
}

/// AES-128-ECB decrypt without padding
fn aes_ecb_decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if data.len() % 16 != 0 {
        return Err("Data length not multiple of 16".into());
    }
    let cipher = Aes128EcbDec::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut buf = data.to_vec();
    cipher
        .decrypt_padded_mut::<NoPadding>(&mut buf)
        .map_err(|e| format!("AES decrypt error: {:?}", e))?;
    Ok(buf)
}

/// Build RC4 lookup table from key_box and meta_key
fn build_rc4_table(key_box: &[u8]) -> [u8; 256] {
    let mut s = [0u8; 256];
    for i in 0..256 {
        s[i] = i as u8;
    }
    let mut j: u8 = 0;
    for i in 0..256 {
        j = j.wrapping_add(key_box[i % key_box.len()].wrapping_add(s[i]));
        s.swap(i, j as usize);
    }
    let mut rc4_table = [0u8; 256];
    for i in 0..256 {
        let si = s[i] as usize;
        let sj = s[(si + i) & 0xFF] as usize;
        rc4_table[i] = s[(si + sj) & 0xFF];
    }
    rc4_table
}

/// Read a little-endian u32 from buffer at pos
fn read_u32_le(buf: &[u8], pos: usize) -> Option<u32> {
    if pos + 4 > buf.len() {
        return None;
    }
    Some(u32::from_le_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]]))
}

/// Parse NCM file: extract music data, metadata, format
pub fn parse_ncm_file(file_path: &Path) -> Result<NcmResult, String> {
    let buffer = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    if buffer.len() < 10 {
        return Err("File too small".into());
    }
    if &buffer[0..8] != MAGIC || buffer[8] != 0 || buffer[9] != 0 {
        return Err("Invalid NCM magic number".into());
    }

    let mut pos = 10usize;

    // 1. Read encrypted key data length
    let encrypted_key_len = read_u32_le(&buffer, pos).ok_or("Failed to read key length")? as usize;
    pos += 4;
    if pos + encrypted_key_len > buffer.len() {
        return Err("Key data exceeds file".into());
    }

    // 2. XOR 0x64
    let mut encrypted_key_data = buffer[pos..pos + encrypted_key_len].to_vec();
    for byte in &mut encrypted_key_data {
        *byte ^= 0x64;
    }
    pos += encrypted_key_len;

    // 3. AES-128-ECB decrypt key
    let decrypted_key_data = aes_ecb_decrypt(&encrypted_key_data, NCM_CORE_KEY)?;

    // 4. Verify "neteasecloudmusic" prefix
    if decrypted_key_data.len() < 17 || &decrypted_key_data[..17] != b"neteasecloudmusic" {
        return Err("Invalid key prefix".into());
    }

    // 5. Read key_box size
    if decrypted_key_data.len() < 21 {
        return Err("Key data too short".into());
    }
    let key_box_size = u32::from_le_bytes([
        decrypted_key_data[17],
        decrypted_key_data[18],
        decrypted_key_data[19],
        decrypted_key_data[20],
    ]) as usize;
    if decrypted_key_data.len() < 21 + key_box_size {
        return Err("Key box data truncated".into());
    }

    // 6. Build key_box XOR meta_key
    let mut key_box = decrypted_key_data[21..21 + key_box_size].to_vec();
    for i in 0..key_box.len() {
        key_box[i] ^= NCM_META_KEY[i % NCM_META_KEY.len()];
    }

    // 7-8. Build RC4 lookup table
    let rc4_table = build_rc4_table(&key_box);

    // 9. Read metadata length
    let metadata_len = read_u32_le(&buffer, pos).ok_or("Failed to read metadata length")? as usize;
    pos += 4;

    // 10-12. Parse metadata
    let mut metadata = NcmMetadata {
        music_name: None,
        artist: None,
        album: None,
        format: None,
        lyric: None,
        extra: serde_json::Value::Null,
    };

    if metadata_len > 0 && pos + metadata_len <= buffer.len() {
        let mut encrypted_meta = buffer[pos..pos + metadata_len].to_vec();
        for byte in &mut encrypted_meta {
            *byte ^= 0x63;
        }

        let b64_prefix = b"163 key(Don't modify):";
        if let Some(b64_start) = find_subsequence(&encrypted_meta, b64_prefix) {
            let b64_data_start = b64_start + b64_prefix.len();
            let b64_str = String::from_utf8_lossy(&encrypted_meta[b64_data_start..]);
            let b64_trimmed = b64_str.trim();

            if let Ok(aes_encrypted_meta) = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                b64_trimmed,
            ) {
                if !aes_encrypted_meta.is_empty() && aes_encrypted_meta.len() % 16 == 0 {
                    if let Ok(meta_plain) = aes_ecb_decrypt(&aes_encrypted_meta, NCM_META_KEY) {
                        let meta_str = String::from_utf8_lossy(&meta_plain);
                        let meta_str = meta_str.trim_end_matches('\0');
                        let json_str = if meta_str.starts_with("music:") {
                            &meta_str[6..]
                        } else {
                            meta_str
                        };
                        if let Ok(parsed) = serde_json::from_str::<NcmMetadata>(json_str) {
                            metadata = parsed;
                        }
                    }
                }
            }
        }
    }
    pos += metadata_len;

    // 13. Skip CRC + image
    if pos + 4 > buffer.len() {
        return Err("File truncated at CRC".into());
    }
    pos += 4; // CRC
    if pos + 4 > buffer.len() {
        return Err("File truncated at image size".into());
    }
    let image_size = read_u32_le(&buffer, pos).ok_or("Failed to read image size")? as usize;
    pos += 4 + image_size;

    // 14. Decrypt audio with RC4 table
    if pos >= buffer.len() {
        return Err("No audio data found".into());
    }
    let mut music_data = buffer[pos..].to_vec();
    for byte in &mut music_data {
        *byte ^= rc4_table[*byte as usize];
    }

    let format = metadata.format.clone().unwrap_or_else(|| "mp3".to_string());

    Ok(NcmResult {
        music_data,
        format,
        metadata,
    })
}

/// Extract NCM info (metadata + cover) without decrypting audio
pub fn extract_ncm_info(file_path: &Path) -> Result<NcmInfo, String> {
    let buffer = fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

    if buffer.len() < 10 || &buffer[0..8] != MAGIC || buffer[8] != 0 || buffer[9] != 0 {
        return Err("Invalid NCM file".into());
    }

    let mut pos = 10usize;

    // Skip encrypted key data
    let encrypted_key_len = read_u32_le(&buffer, pos).ok_or("Failed to read key length")? as usize;
    pos += 4 + encrypted_key_len;

    // Read metadata
    let metadata_len = read_u32_le(&buffer, pos).ok_or("Failed to read metadata length")? as usize;
    pos += 4;

    let mut metadata = NcmMetadata {
        music_name: None,
        artist: None,
        album: None,
        format: None,
        lyric: None,
        extra: serde_json::Value::Null,
    };

    if metadata_len > 0 && pos + metadata_len <= buffer.len() {
        let mut encrypted_meta = buffer[pos..pos + metadata_len].to_vec();
        for byte in &mut encrypted_meta {
            *byte ^= 0x63;
        }

        let b64_prefix = b"163 key(Don't modify):";
        if let Some(b64_start) = find_subsequence(&encrypted_meta, b64_prefix) {
            let b64_data_start = b64_start + b64_prefix.len();
            let b64_str = String::from_utf8_lossy(&encrypted_meta[b64_data_start..]);
            let b64_trimmed = b64_str.trim();

            if let Ok(aes_encrypted_meta) = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                b64_trimmed,
            ) {
                if !aes_encrypted_meta.is_empty() && aes_encrypted_meta.len() % 16 == 0 {
                    if let Ok(meta_plain) = aes_ecb_decrypt(&aes_encrypted_meta, NCM_META_KEY) {
                        let meta_str = String::from_utf8_lossy(&meta_plain);
                        let meta_str = meta_str.trim_end_matches('\0');
                        let json_str = if meta_str.starts_with("music:") {
                            &meta_str[6..]
                        } else {
                            meta_str
                        };
                        if let Ok(parsed) = serde_json::from_str::<NcmMetadata>(json_str) {
                            metadata = parsed;
                        }
                    }
                }
            }
        }
    }
    pos += metadata_len;

    // Read cover image
    let mut cover_image = None;
    let mut cover_format = None;

    if pos + 4 <= buffer.len() {
        pos += 4; // CRC
        if pos + 4 <= buffer.len() {
            let image_size = read_u32_le(&buffer, pos).unwrap_or(0) as usize;
            pos += 4;
            if image_size > 0 && pos + image_size <= buffer.len() {
                let img_data = &buffer[pos..pos + image_size];
                let format = if img_data.len() >= 2 && img_data[0] == 0x89 && img_data[1] == 0x50 {
                    "png"
                } else {
                    "jpg"
                };
                cover_image = Some(img_data.to_vec());
                cover_format = Some(format.to_string());
            }
        }
    }

    Ok(NcmInfo {
        metadata,
        cover_image,
        cover_format,
    })
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
