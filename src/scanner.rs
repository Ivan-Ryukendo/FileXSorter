//! Scanner module - Directory walking and file discovery
//!
//! This module handles recursive/non-recursive directory traversal
//! and file metadata collection.

use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024 * 1024;
const MAX_PARALLEL_THREADS: usize = 8;

/// Represents a scanned file with metadata
#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub hash: Option<String>,
}

impl FileEntry {
    pub fn new(path: PathBuf, name: String, size: u64) -> Self {
        Self {
            path,
            name,
            size,
            hash: None,
        }
    }
}

/// A group of duplicate files (same hash)
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub hash: String,
    pub files: Vec<FileEntry>,
    pub total_size: u64,
    pub wasted_size: u64,
}

/// Progress tracking for scan operations
#[derive(Debug, Clone, Default)]
pub struct ScanProgress {
    pub phase: String,
    pub total_files: usize,
    pub processed_files: usize,
    pub current_file: String,
}

/// Result of a duplicate scan
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub total_files: usize,
    pub total_size: u64,
    pub duplicate_groups: Vec<DuplicateGroup>,
    pub total_duplicates: usize,
    pub wasted_space: u64,
    pub errors: Vec<String>,
}

/// Scanner configuration
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    pub recursive: bool,
    pub min_size: u64,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            recursive: true,
            min_size: 1,
        }
    }
}

/// The main scanner struct
pub struct Scanner {
    config: ScannerConfig,
    cancel_flag: Arc<AtomicBool>,
    progress_total: Arc<AtomicUsize>,
    progress_current: Arc<AtomicUsize>,
}

impl Scanner {
    pub fn new(config: ScannerConfig) -> Self {
        Self {
            config,
            cancel_flag: Arc::new(AtomicBool::new(false)),
            progress_total: Arc::new(AtomicUsize::new(0)),
            progress_current: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get a cancellation handle
    pub fn get_cancel_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.cancel_flag)
    }

    /// Get progress info
    pub fn get_progress(&self) -> (usize, usize) {
        (
            self.progress_current.load(Ordering::Relaxed),
            self.progress_total.load(Ordering::Relaxed),
        )
    }

    /// Cancel the current scan
    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }

    /// Reset cancellation flag
    pub fn reset(&self) {
        self.cancel_flag.store(false, Ordering::Relaxed);
        self.progress_current.store(0, Ordering::Relaxed);
        self.progress_total.store(0, Ordering::Relaxed);
    }

    /// Scan a directory for duplicate files
    pub fn scan_directory(&self, path: &Path) -> ScanResult {
        self.scan_directories_with_progress(
            &[path.to_path_buf()],
            &self.progress_current,
            &self.progress_total,
            &self.cancel_flag,
        )
    }

    /// Scan multiple directories for duplicate files
    pub fn scan_directories(&self, paths: &[PathBuf]) -> ScanResult {
        self.scan_directories_with_progress(
            paths,
            &self.progress_current,
            &self.progress_total,
            &self.cancel_flag,
        )
    }

    /// Scan multiple directories for duplicate files with external progress tracking
    pub fn scan_directories_with_progress(
        &self,
        paths: &[PathBuf],
        progress_current: &AtomicUsize,
        progress_total: &AtomicUsize,
        cancel_flag: &AtomicBool,
    ) -> ScanResult {
        progress_current.store(0, Ordering::Relaxed);
        progress_total.store(0, Ordering::Relaxed);

        let mut result = ScanResult::default();

        // Collect files from all directories
        let mut files = Vec::new();
        for path in paths.iter() {
            if cancel_flag.load(Ordering::Relaxed) {
                return result;
            }
            let mut dir_files =
                self.collect_files_with_cancel(path, cancel_flag, &mut result.errors);
            files.append(&mut dir_files);
        }

        if cancel_flag.load(Ordering::Relaxed) {
            return result;
        }

        result.total_files = files.len();
        result.total_size = files.iter().map(|f| f.size).sum();

        let size_groups = self.group_by_size(files);

        let potential_duplicates: Vec<FileEntry> = size_groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .flat_map(|(_, files)| files)
            .collect();

        if potential_duplicates.is_empty() || cancel_flag.load(Ordering::Relaxed) {
            return result;
        }

        progress_total.store(potential_duplicates.len(), Ordering::Relaxed);
        progress_current.store(0, Ordering::Relaxed);

        let hashed_files = self.hash_files(
            potential_duplicates,
            progress_current,
            cancel_flag,
            &mut result.errors,
        );

        if cancel_flag.load(Ordering::Relaxed) {
            return result;
        }

        let hash_groups = self.group_by_hash(hashed_files);

        for (hash, files) in hash_groups {
            if files.len() > 1 {
                let total_size: u64 = files.iter().map(|f| f.size).sum();
                let wasted_size = total_size - files[0].size;

                result.total_duplicates += files.len() - 1;
                result.wasted_space += wasted_size;

                result.duplicate_groups.push(DuplicateGroup {
                    hash,
                    files,
                    total_size,
                    wasted_size,
                });
            }
        }

        result
            .duplicate_groups
            .sort_by(|a, b| b.wasted_size.cmp(&a.wasted_size));

        result
    }

    /// Collect all files from directory with external cancel flag
    fn collect_files_with_cancel(
        &self,
        path: &Path,
        cancel_flag: &AtomicBool,
        errors: &mut Vec<String>,
    ) -> Vec<FileEntry> {
        let mut files = Vec::new();

        let walker = if self.config.recursive {
            WalkDir::new(path).follow_links(false)
        } else {
            WalkDir::new(path).max_depth(1).follow_links(false)
        };

        for entry in walker.into_iter().filter_map(|e| e.ok()) {
            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            let entry_path = entry.path();

            if entry_path.is_file() {
                match fs::metadata(entry_path) {
                    Ok(metadata) => {
                        let size = metadata.len();
                        if size >= self.config.min_size && size <= MAX_FILE_SIZE {
                            let name = entry_path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();

                            files.push(FileEntry::new(entry_path.to_path_buf(), name, size));
                        }
                    }
                    Err(e) => {
                        errors.push(format!("Failed to read {}: {}", entry_path.display(), e));
                    }
                }
            }
        }

        files
    }

    /// Hash files with thread limit and progress tracking
    fn hash_files(
        &self,
        files: Vec<FileEntry>,
        progress_current: &AtomicUsize,
        cancel_flag: &AtomicBool,
        errors: &mut Vec<String>,
    ) -> Vec<FileEntry> {
        let results: Vec<Result<FileEntry, String>> = files
            .par_iter()
            .map(|file| {
                if cancel_flag.load(Ordering::Relaxed) {
                    return Err("Cancelled".to_string());
                }

                match compute_file_hash(&file.path) {
                    Ok(hash) => {
                        let mut hashed_file = file.clone();
                        hashed_file.hash = Some(hash);
                        progress_current.fetch_add(1, Ordering::Relaxed);
                        Ok(hashed_file)
                    }
                    Err(e) => Err(format!("Failed to hash {}: {}", file.path.display(), e)),
                }
            })
            .collect();

        let mut hashed_files = Vec::new();
        for result in results {
            match result {
                Ok(file) => hashed_files.push(file),
                Err(e) if e != "Cancelled" => errors.push(e),
                _ => {}
            }
        }

        hashed_files
    }

    /// Group files by size
    fn group_by_size(&self, files: Vec<FileEntry>) -> HashMap<u64, Vec<FileEntry>> {
        let mut groups: HashMap<u64, Vec<FileEntry>> = HashMap::new();

        for file in files {
            groups.entry(file.size).or_default().push(file);
        }

        groups
    }

    /// Group files by hash
    fn group_by_hash(&self, files: Vec<FileEntry>) -> HashMap<String, Vec<FileEntry>> {
        let mut groups: HashMap<String, Vec<FileEntry>> = HashMap::new();

        for file in files {
            if let Some(ref hash) = file.hash {
                groups.entry(hash.clone()).or_default().push(file);
            }
        }

        groups
    }
}

/// Compute SHA-256 hash of a file with chunked reading and size limit
fn compute_file_hash(path: &Path) -> std::io::Result<String> {
    let metadata = fs::metadata(path)?;

    if metadata.len() > MAX_FILE_SIZE {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "File too large ({} > {} bytes)",
                metadata.len(),
                MAX_FILE_SIZE
            ),
        ));
    }

    const BUFFER_SIZE: usize = 1024 * 1024;

    let file = fs::File::open(path)?;
    let mut reader = BufReader::with_capacity(BUFFER_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; BUFFER_SIZE];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Format bytes into human-readable size
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }
}
