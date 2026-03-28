use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use sha1::Digest;

/// Scanned file info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: PathBuf,
    pub size: u64,
    pub md5: String,
    pub sha1: String,
}

/// Folder scanner for finding complete torrents
pub struct FolderScanner;

impl FolderScanner {
    /// Scan a folder for complete files
    pub fn scan_folder(path: &Path, extensions: Option<Vec<&str>>) -> Result<Vec<ScannedFile>> {
        let mut files = Vec::new();
        let exts = extensions.unwrap_or_else(|| vec!["torrent", "iso", "zip", "tar", "gz"]);

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if let Some(ext_str) = ext.to_str() {
                        if exts.contains(&ext_str) {
                            if let Ok(file_info) = Self::calculate_hashes(path) {
                                files.push(file_info);
                            }
                        }
                    }
                }
            }
        }

        tracing::info!(
            "Scanned folder {:?} and found {} files",
            path,
            files.len()
        );
        Ok(files)
    }

    /// Calculate MD5 and SHA1 hashes for a file
    pub fn calculate_hashes(path: &Path) -> Result<ScannedFile> {
        use std::fs::File;
        use std::io::Read;

        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let size = metadata.len();

        let mut hasher_md5 = md5::Context::new();
        let mut hasher_sha1 = sha1::Sha1::new();
        let mut reader = File::open(path)?;
        let mut buffer = [0; 8192];

        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher_md5.consume(&buffer[..n]);
            hasher_sha1.update(&buffer[..n]);
        }

        let md5_hash = format!("{:x}", hasher_md5.compute());
        let sha1_hash = format!("{:x}", hasher_sha1.finalize());

        Ok(ScannedFile {
            path: path.to_path_buf(),
            size,
            md5: md5_hash,
            sha1: sha1_hash,
        })
    }

    /// Find matching torrent files for a given file hash
    pub fn find_matching_torrents(
        torrent_files: &[ScannedFile],
        target_hash: &str,
    ) -> Vec<ScannedFile> {
        torrent_files
            .iter()
            .filter(|f| f.sha1.to_lowercase() == target_hash.to_lowercase() 
                || f.md5.to_lowercase() == target_hash.to_lowercase())
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_matching_torrents() {
        let files = vec![
            ScannedFile {
                path: PathBuf::from("test1.torrent"),
                size: 1024,
                md5: "abc123".to_string(),
                sha1: "def456".to_string(),
            },
            ScannedFile {
                path: PathBuf::from("test2.torrent"),
                size: 2048,
                md5: "ghi789".to_string(),
                sha1: "jkl012".to_string(),
            },
        ];

        let matches = FolderScanner::find_matching_torrents(&files, "abc123");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].md5, "abc123");
    }
}
