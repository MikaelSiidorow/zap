use crate::store::{enforce_retention, open_db, upsert_entry, upsert_image_entry};
use log::{debug, warn};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::Duration;

pub struct ClipboardConfig {
    pub max_age_days: u32,
    pub max_entries: usize,
    pub poll_interval_ms: u64,
}

impl Default for ClipboardConfig {
    fn default() -> Self {
        Self {
            max_age_days: 30,
            max_entries: 1000,
            poll_interval_ms: 500,
        }
    }
}

pub fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn sha256_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn looks_sensitive(text: &str) -> bool {
    let trimmed = text.trim();

    // Private keys
    if trimmed.starts_with("-----BEGIN PRIVATE KEY")
        || trimmed.starts_with("-----BEGIN RSA PRIVATE KEY")
        || trimmed.starts_with("-----BEGIN OPENSSH PRIVATE KEY")
    {
        return true;
    }

    // OTP URIs
    if trimmed.starts_with("otpauth://") {
        return true;
    }

    // OTP-like codes (6-8 digits only, nothing else)
    if trimmed.len() >= 6 && trimmed.len() <= 8 && trimmed.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    false
}

/// 10 MB limit for RGBA image data
const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;

pub fn spawn_monitor(db_path: PathBuf, blob_dir: PathBuf, config: ClipboardConfig) {
    std::thread::spawn(move || {
        let conn = match open_db(&db_path) {
            Ok(c) => c,
            Err(e) => {
                warn!("clipboard monitor: failed to open db: {e}");
                return;
            }
        };
        let mut last_hash = String::new();
        debug!("clipboard monitor started");
        loop {
            std::thread::sleep(Duration::from_millis(config.poll_interval_ms));
            let Ok(mut clipboard) = arboard::Clipboard::new() else {
                continue;
            };

            // Try text first
            if let Ok(text) = clipboard.get_text() {
                if !text.trim().is_empty() {
                    let hash = sha256_hex(&text);
                    if hash != last_hash {
                        last_hash = hash.clone();
                        if looks_sensitive(&text) {
                            debug!("clipboard monitor: skipping sensitive content");
                            continue;
                        }
                        let _ = upsert_entry(&conn, &text, &hash);
                        cleanup_blobs(&conn, &config);
                        continue;
                    }
                    continue;
                }
            }

            // Try image
            if let Ok(img) = clipboard.get_image() {
                let rgba_bytes = img.bytes.as_ref();
                if rgba_bytes.len() > MAX_IMAGE_BYTES {
                    debug!(
                        "clipboard monitor: skipping large image ({}MB)",
                        rgba_bytes.len() / 1024 / 1024
                    );
                    continue;
                }
                let hash = sha256_bytes(rgba_bytes);
                if hash == last_hash {
                    continue;
                }
                last_hash = hash.clone();

                // Use first 16 chars of hash for filename
                let filename = format!("{}.png", &hash[..16]);
                let blob_path = blob_dir.join(&filename);

                if !blob_path.exists() {
                    if let Some(img_buf) = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                        img.width as u32,
                        img.height as u32,
                        rgba_bytes.to_vec(),
                    ) {
                        if let Err(e) = img_buf.save(&blob_path) {
                            warn!("clipboard monitor: failed to save PNG: {e}");
                            continue;
                        }
                    } else {
                        warn!("clipboard monitor: failed to create ImageBuffer");
                        continue;
                    }
                }

                let content = format!("Image ({}x{})", img.width, img.height);
                let _ = upsert_image_entry(
                    &conn,
                    &content,
                    &hash,
                    blob_path.to_str().unwrap_or_default(),
                );
                cleanup_blobs(&conn, &config);
            }
        }
    });
}

fn cleanup_blobs(conn: &rusqlite::Connection, config: &ClipboardConfig) {
    match enforce_retention(conn, config.max_age_days, config.max_entries) {
        Ok(blob_paths) => {
            for path in blob_paths {
                if let Err(e) = std::fs::remove_file(&path) {
                    debug!("clipboard monitor: failed to remove blob {path}: {e}");
                }
            }
        }
        Err(e) => {
            debug!("clipboard monitor: retention error: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex("hello");
        assert_eq!(hash.len(), 64);
        // Deterministic
        assert_eq!(hash, sha256_hex("hello"));
        assert_ne!(hash, sha256_hex("world"));
    }

    #[test]
    fn test_sha256_bytes() {
        let hash = sha256_bytes(b"hello");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, sha256_bytes(b"hello"));
        assert_ne!(hash, sha256_bytes(b"world"));
    }

    #[test]
    fn test_looks_sensitive() {
        assert!(looks_sensitive("-----BEGIN PRIVATE KEY-----\nfoo"));
        assert!(looks_sensitive("-----BEGIN RSA PRIVATE KEY-----\nfoo"));
        assert!(looks_sensitive("-----BEGIN OPENSSH PRIVATE KEY-----\nfoo"));
        assert!(looks_sensitive("otpauth://totp/Example:alice"));
        assert!(looks_sensitive("123456"));
        assert!(looks_sensitive("12345678"));
        assert!(!looks_sensitive("12345")); // too short
        assert!(!looks_sensitive("123456789")); // too long
        assert!(!looks_sensitive("hello world"));
        assert!(!looks_sensitive("some normal text"));
    }
}
