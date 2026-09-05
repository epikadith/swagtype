//! Cross-platform configuration and data directory helpers.
//!
//! Uses the `directories` crate to resolve OS-appropriate paths:
//! - Linux: `~/.local/share/swagtype/`
//! - macOS: `~/Library/Application Support/com.swagtype.swagtype/`
//! - Windows: `C:\Users\<user>\AppData\Roaming\swagtype\`

use directories::ProjectDirs;
use std::path::PathBuf;

/// Qualifier, org, and app used by the `directories` crate.
const QUALIFIER: &str = "com";
const ORG: &str = "swagtype";
const APP: &str = "swagtype";

/// Returns the project data directory path.
///
/// Returns `None` only if the home directory cannot be determined (extremely
/// rare on any supported OS).
pub fn data_dir() -> Option<PathBuf> {
    ProjectDirs::from(QUALIFIER, ORG, APP).map(|dirs| dirs.data_dir().to_path_buf())
}

/// Ensures the project data directory exists, creating it if necessary.
///
/// Returns the path on success.
pub fn ensure_data_dir() -> std::io::Result<PathBuf> {
    let dir = data_dir().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine home directory",
        )
    })?;
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_dir_contains_swagtype() {
        let dir = data_dir().expect("should resolve data dir in test environment");
        let path_str = dir.to_string_lossy();
        assert!(
            path_str.contains("swagtype"),
            "data_dir should contain 'swagtype', got: {path_str}"
        );
    }

    #[test]
    fn ensure_data_dir_creates_directory() {
        // This actually creates the directory on the filesystem,
        // which is acceptable for an integration-style test.
        let dir = ensure_data_dir().expect("should create data dir");
        assert!(dir.exists(), "data dir should exist after ensure_data_dir");
        assert!(dir.is_dir(), "data dir should be a directory");
    }
}
