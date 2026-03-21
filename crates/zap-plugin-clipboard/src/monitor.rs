use crate::store::{enforce_retention, open_db, upsert_entry};
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

pub fn spawn_monitor(db_path: PathBuf, config: ClipboardConfig) {
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
            let Ok(text) = clipboard.get_text() else {
                continue;
            };
            if text.trim().is_empty() {
                continue;
            }
            let hash = sha256_hex(&text);
            if hash == last_hash {
                continue;
            }
            last_hash = hash.clone();
            if looks_sensitive(&text) {
                debug!("clipboard monitor: skipping sensitive content");
                continue;
            }
            let _ = upsert_entry(&conn, &text, &hash);
            let _ = enforce_retention(&conn, config.max_age_days, config.max_entries);
        }
    });
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
