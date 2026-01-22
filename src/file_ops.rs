//! File operations module - Delete and Move functionality
//!
//! This module handles file deletion and moving operations
//! with proper error handling and logging.

use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Result of a file operation
#[derive(Debug, Clone)]
pub enum OperationResult {
    Success(String),
    Error(String),
}

/// Log entry for file operations
#[derive(Debug, Clone)]
pub struct OperationLog {
    pub operation: String,
    pub source: PathBuf,
    pub destination: Option<PathBuf>,
    pub success: bool,
    pub message: String,
}

/// File operations handler
pub struct FileOperations {
    logs: Vec<OperationLog>,
}

impl Default for FileOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOperations {
    pub fn new() -> Self {
        Self { logs: Vec::new() }
    }

    /// Get operation logs
    pub fn get_logs(&self) -> &[OperationLog] {
        &self.logs
    }

    /// Clear operation logs
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }

    /// Delete a file
    pub fn delete_file(&mut self, path: &Path) -> OperationResult {
        match fs::remove_file(path) {
            Ok(()) => {
                let msg = format!("Deleted: {}", path.display());
                self.logs.push(OperationLog {
                    operation: "DELETE".to_string(),
                    source: path.to_path_buf(),
                    destination: None,
                    success: true,
                    message: msg.clone(),
                });
                OperationResult::Success(msg)
            }
            Err(e) => {
                let msg = format!("Failed to delete {}: {}", path.display(), e);
                self.logs.push(OperationLog {
                    operation: "DELETE".to_string(),
                    source: path.to_path_buf(),
                    destination: None,
                    success: false,
                    message: msg.clone(),
                });
                OperationResult::Error(msg)
            }
        }
    }

    /// Delete multiple files
    pub fn delete_files(&mut self, paths: &[PathBuf]) -> Vec<OperationResult> {
        paths.iter().map(|p| self.delete_file(p)).collect()
    }

    /// Move a file to a destination directory
    pub fn move_file(&mut self, source: &Path, dest_dir: &Path) -> OperationResult {
        // Ensure destination directory exists (handle race condition directly)
        match fs::create_dir_all(dest_dir) {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => {
                let msg = format!("Failed to create directory {}: {}", dest_dir.display(), e);
                return OperationResult::Error(msg);
            }
        }

        // Build destination path
        let file_name = source.file_name().unwrap_or_default();
        let mut dest_path = dest_dir.join(file_name);

        // Handle filename conflicts - generate unique path if file exists
        if dest_path.exists() {
            dest_path = generate_unique_path(&dest_path);
        }

        match fs::rename(source, &dest_path) {
            Ok(()) => {
                let msg = format!("Moved: {} -> {}", source.display(), dest_path.display());
                self.logs.push(OperationLog {
                    operation: "MOVE".to_string(),
                    source: source.to_path_buf(),
                    destination: Some(dest_path),
                    success: true,
                    message: msg.clone(),
                });
                OperationResult::Success(msg)
            }
            Err(e) => {
                // Try copy + delete if rename fails (cross-drive moves)
                match fs::copy(source, &dest_path) {
                    Ok(_) => match fs::remove_file(source) {
                        Ok(()) => {
                            let msg =
                                format!("Moved: {} -> {}", source.display(), dest_path.display());
                            self.logs.push(OperationLog {
                                operation: "MOVE".to_string(),
                                source: source.to_path_buf(),
                                destination: Some(dest_path),
                                success: true,
                                message: msg.clone(),
                            });
                            OperationResult::Success(msg)
                        }
                        Err(del_err) => {
                            // Copy succeeded but delete failed - clean up
                            let _ = fs::remove_file(&dest_path);
                            let msg = format!(
                                "Failed to complete move of {}: {}",
                                source.display(),
                                del_err
                            );
                            self.logs.push(OperationLog {
                                operation: "MOVE".to_string(),
                                source: source.to_path_buf(),
                                destination: Some(dest_path),
                                success: false,
                                message: msg.clone(),
                            });
                            OperationResult::Error(msg)
                        }
                    },
                    Err(_) => {
                        let msg = format!("Failed to move {}: {}", source.display(), e);
                        self.logs.push(OperationLog {
                            operation: "MOVE".to_string(),
                            source: source.to_path_buf(),
                            destination: Some(dest_path),
                            success: false,
                            message: msg.clone(),
                        });
                        OperationResult::Error(msg)
                    }
                }
            }
        }
    }

    /// Move multiple files to a destination directory
    pub fn move_files(&mut self, sources: &[PathBuf], dest_dir: &Path) -> Vec<OperationResult> {
        sources
            .iter()
            .map(|p| self.move_file(p, dest_dir))
            .collect()
    }
}

/// Generate a unique path by appending a number
fn generate_unique_path(path: &Path) -> PathBuf {
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
    let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let parent = path.parent().unwrap_or(Path::new("."));

    let mut counter = 1;
    loop {
        let new_name = if extension.is_empty() {
            format!("{}_{}", stem, counter)
        } else {
            format!("{}_{}.{}", stem, counter, extension)
        };

        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;

        // Safety limit
        if counter > 10000 {
            return parent.join(format!("{}_{}", stem, uuid_simple()));
        }
    }
}

/// Generate a cryptographically secure unique ID
fn uuid_simple() -> String {
    Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_unique_path() {
        let path = Path::new("/tmp/test.txt");
        let unique = generate_unique_path(path);
        assert!(unique.to_string_lossy().contains("test_1.txt"));
    }
}
