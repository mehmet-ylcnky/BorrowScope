//! Utility functions for file and workspace management
//!
//! This module provides safe temporary workspace management with RAII pattern,
//! atomic file operations, and various file system utilities.

use anyhow::{Context, Result};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use walkdir::WalkDir;

/// Manages a temporary workspace for instrumented code
///
/// Uses RAII pattern - automatically cleans up on drop unless persisted.
#[allow(dead_code)]
pub struct TempWorkspace {
    dir: TempDir,
    original_path: PathBuf,
}

#[allow(dead_code)]
impl TempWorkspace {
    /// Create a new temporary workspace
    pub fn new() -> Result<Self> {
        let dir = TempDir::new().context("Failed to create temporary directory")?;

        Ok(Self {
            dir,
            original_path: PathBuf::new(),
        })
    }

    /// Create with a specific prefix
    pub fn with_prefix(prefix: &str) -> Result<Self> {
        let dir = TempDir::with_prefix(prefix).context("Failed to create temporary directory")?;

        Ok(Self {
            dir,
            original_path: PathBuf::new(),
        })
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Get the original source path
    pub fn original_path(&self) -> &Path {
        &self.original_path
    }

    /// Copy an entire project to the temporary workspace
    pub fn copy_project(&mut self, source: &Path) -> Result<PathBuf> {
        if !source.exists() {
            anyhow::bail!("Source path does not exist: {:?}", source);
        }

        self.original_path = source.to_path_buf();

        let dest = self.dir.path().join("project");

        if source.is_file() {
            // Single file - copy just the file
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(source, &dest)?;
        } else {
            // Directory - copy recursively
            copy_dir_recursive(source, &dest)?;
        }

        Ok(dest)
    }

    /// Copy only specific files matching patterns
    pub fn copy_filtered(&mut self, source: &Path, patterns: &[String]) -> Result<PathBuf> {
        self.original_path = source.to_path_buf();
        let dest = self.dir.path().join("project");

        fs::create_dir_all(&dest)?;

        for entry in WalkDir::new(source)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Always allow directories
            if path.is_dir() {
                continue;
            }

            // Check if file matches patterns
            if !should_include_file(path, patterns) {
                continue;
            }

            let relative = path.strip_prefix(source)?;
            let dest_path = dest.join(relative);

            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::copy(path, dest_path)?;
        }

        Ok(dest)
    }

    /// Create a subdirectory in the workspace
    pub fn create_subdir(&self, name: &str) -> Result<PathBuf> {
        let path = self.dir.path().join(name);
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    /// Write a file to the workspace
    pub fn write_file(&self, relative_path: &str, content: &str) -> Result<PathBuf> {
        let path = self.dir.path().join(relative_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, content)?;
        Ok(path)
    }

    /// Write bytes to a file in the workspace
    pub fn write_bytes(&self, relative_path: &str, content: &[u8]) -> Result<PathBuf> {
        let path = self.dir.path().join(relative_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, content)?;
        Ok(path)
    }

    /// Read a file from the workspace
    pub fn read_file(&self, relative_path: &str) -> Result<String> {
        let path = self.dir.path().join(relative_path);
        fs::read_to_string(&path).context("Failed to read file from workspace")
    }

    /// Read bytes from a file in the workspace
    pub fn read_bytes(&self, relative_path: &str) -> Result<Vec<u8>> {
        let path = self.dir.path().join(relative_path);
        fs::read(&path).context("Failed to read file from workspace")
    }

    /// Check if a file exists in the workspace
    pub fn exists(&self, relative_path: &str) -> bool {
        self.dir.path().join(relative_path).exists()
    }

    /// List all files in the workspace
    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(self.dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                if let Ok(relative) = entry.path().strip_prefix(self.dir.path()) {
                    files.push(relative.to_path_buf());
                }
            }
        }

        Ok(files)
    }

    /// List all directories in the workspace
    pub fn list_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = Vec::new();

        for entry in WalkDir::new(self.dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_dir() && entry.path() != self.dir.path() {
                if let Ok(relative) = entry.path().strip_prefix(self.dir.path()) {
                    dirs.push(relative.to_path_buf());
                }
            }
        }

        Ok(dirs)
    }

    /// Get the size of the workspace in bytes
    pub fn size(&self) -> Result<u64> {
        dir_size(self.dir.path())
    }

    /// Get the number of files in the workspace
    pub fn file_count(&self) -> Result<usize> {
        Ok(self.list_files()?.len())
    }

    /// Explicitly cleanup the workspace
    /// (Note: cleanup happens automatically on drop)
    pub fn cleanup(self) -> Result<()> {
        // TempDir handles cleanup on drop
        Ok(())
    }

    /// Keep the temporary directory (don't delete on drop)
    pub fn persist(self) -> Result<PathBuf> {
        Ok(self.dir.keep())
    }

    /// Copy workspace contents back to original location
    pub fn copy_back(&self) -> Result<()> {
        if self.original_path.as_os_str().is_empty() {
            anyhow::bail!("No original path set");
        }

        let project_dir = self.dir.path().join("project");
        if !project_dir.exists() {
            anyhow::bail!("Project directory not found in workspace");
        }

        copy_dir_recursive(&project_dir, &self.original_path)?;
        Ok(())
    }
}

/// Copy a directory recursively
#[allow(dead_code)]
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else if ty.is_file() {
            fs::copy(&src_path, &dst_path)?;
        } else if ty.is_symlink() {
            // Handle symlinks
            let target = fs::read_link(&src_path)?;
            #[cfg(unix)]
            {
                let _ = std::os::unix::fs::symlink(&target, &dst_path);
            }
            #[cfg(windows)]
            {
                if target.is_dir() {
                    let _ = std::os::windows::fs::symlink_dir(&target, &dst_path);
                } else {
                    let _ = std::os::windows::fs::symlink_file(&target, &dst_path);
                }
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn should_include(path: &Path, patterns: &[String]) -> bool {
    // Skip hidden files and directories
    if path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
    {
        return false;
    }

    // Skip target directory
    let path_str = path.to_string_lossy();
    if path_str.contains("/target/") || path_str.contains("\\target\\") {
        return false;
    }

    // If no patterns, include everything
    if patterns.is_empty() {
        return true;
    }

    // Check if path matches any pattern
    for pattern in patterns {
        if path_str.contains(pattern) {
            return true;
        }

        // Also check just the filename
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.contains(pattern) {
                return true;
            }
        }
    }

    false
}

#[allow(dead_code)]
fn should_include_file(path: &Path, patterns: &[String]) -> bool {
    // Skip hidden files
    if path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
    {
        return false;
    }

    // If no patterns, include all files
    if patterns.is_empty() {
        return true;
    }

    let path_str = path.to_string_lossy();

    // Check if path matches any pattern
    for pattern in patterns {
        if path_str.contains(pattern) {
            return true;
        }

        // Also check just the filename
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.contains(pattern) {
                return true;
            }
        }
    }

    false
}

/// Atomic file writer - writes to temp file then renames
#[allow(dead_code)]
pub struct AtomicWriter {
    temp_path: PathBuf,
    #[allow(dead_code)]
    final_path: PathBuf,
}

#[allow(dead_code)]
impl AtomicWriter {
    /// Create a new atomic writer
    pub fn new(path: PathBuf) -> Result<Self> {
        let temp_path = path.with_extension("tmp");

        Ok(Self {
            temp_path,
            final_path: path,
        })
    }

    /// Write string content atomically
    pub fn write(&self, content: &str) -> Result<()> {
        // Write to temporary file
        fs::write(&self.temp_path, content).context("Failed to write to temporary file")?;

        // Atomically rename to final path
        fs::rename(&self.temp_path, &self.final_path).context("Failed to rename temporary file")?;

        Ok(())
    }

    /// Write bytes atomically
    pub fn write_bytes(&self, content: &[u8]) -> Result<()> {
        fs::write(&self.temp_path, content)?;
        fs::rename(&self.temp_path, &self.final_path)?;
        Ok(())
    }
}

impl Drop for AtomicWriter {
    fn drop(&mut self) {
        // Clean up temp file if it still exists
        let _ = fs::remove_file(&self.temp_path);
    }
}

/// Find Cargo.toml in current or parent directories
#[allow(dead_code)]
pub fn find_cargo_toml(start: &Path) -> Option<PathBuf> {
    let mut current = start;

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            return Some(cargo_toml);
        }

        current = current.parent()?;
    }
}

/// Check if path is a Rust project
#[allow(dead_code)]
pub fn is_rust_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
}

/// Get project name from Cargo.toml
#[allow(dead_code)]
pub fn get_project_name(path: &Path) -> Result<String> {
    let cargo_toml = find_cargo_toml(path).ok_or_else(|| anyhow::anyhow!("No Cargo.toml found"))?;

    let content = fs::read_to_string(cargo_toml)?;
    let toml: toml::Value = toml::from_str(&content)?;

    let name = toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("No package name in Cargo.toml"))?;

    Ok(name.to_string())
}

/// Calculate directory size
#[allow(dead_code)]
pub fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0;

    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_file() {
            total += entry.metadata()?.len();
        }
    }

    Ok(total)
}

/// Format bytes as human-readable string
#[allow(dead_code)]
pub fn format_bytes(bytes: u64) -> String {
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

/// Safe file removal with retry
#[allow(dead_code)]
pub fn remove_file_safe(path: &Path) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_MS: u64 = 100;

    for attempt in 0..MAX_RETRIES {
        match fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(_e) if attempt < MAX_RETRIES - 1 => {
                std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

/// Safe directory removal with retry
#[allow(dead_code)]
pub fn remove_dir_safe(path: &Path) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_MS: u64 = 100;

    for attempt in 0..MAX_RETRIES {
        match fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(_e) if attempt < MAX_RETRIES - 1 => {
                std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

/// Copy a single file with error handling
#[allow(dead_code)]
pub fn copy_file_safe(src: &Path, dst: &Path) -> Result<u64> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::copy(src, dst)
        .with_context(|| format!("Failed to copy {} to {}", src.display(), dst.display()))
}

/// Check if two paths point to the same file
#[allow(dead_code)]
pub fn same_file(path1: &Path, path2: &Path) -> Result<bool> {
    let meta1 = fs::metadata(path1)?;
    let meta2 = fs::metadata(path2)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(meta1.dev() == meta2.dev() && meta1.ino() == meta2.ino())
    }

    #[cfg(not(unix))]
    {
        // Fallback: compare canonical paths
        Ok(path1.canonicalize()? == path2.canonicalize()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temp_workspace_creation() {
        let workspace = TempWorkspace::new().unwrap();
        assert!(workspace.path().exists());
    }

    #[test]
    fn test_temp_workspace_with_prefix() {
        let workspace = TempWorkspace::with_prefix("borrowscope").unwrap();
        assert!(workspace.path().exists());
        assert!(workspace.path().to_string_lossy().contains("borrowscope"));
    }

    #[test]
    fn test_temp_workspace_cleanup() {
        let path = {
            let workspace = TempWorkspace::new().unwrap();
            workspace.path().to_path_buf()
        };

        // After drop, directory should be cleaned up
        assert!(!path.exists());
    }

    #[test]
    fn test_copy_project_file() {
        let temp_src = TempDir::new().unwrap();
        let src_file = temp_src.path().join("test.txt");
        fs::write(&src_file, "test content").unwrap();

        let mut workspace = TempWorkspace::new().unwrap();
        let dest = workspace.copy_project(&src_file).unwrap();

        assert!(dest.exists());
        let content = fs::read_to_string(&dest).unwrap();
        assert_eq!(content, "test content");
    }

    #[test]
    fn test_copy_project_directory() {
        let temp_src = TempDir::new().unwrap();
        fs::write(temp_src.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_src.path().join("file2.txt"), "content2").unwrap();

        let mut workspace = TempWorkspace::new().unwrap();
        let dest = workspace.copy_project(temp_src.path()).unwrap();

        assert!(dest.exists());
        assert!(dest.join("file1.txt").exists());
        assert!(dest.join("file2.txt").exists());
    }

    #[test]
    fn test_write_and_read_file() {
        let workspace = TempWorkspace::new().unwrap();

        workspace.write_file("test.txt", "hello world").unwrap();
        let content = workspace.read_file("test.txt").unwrap();

        assert_eq!(content, "hello world");
    }

    #[test]
    fn test_write_and_read_bytes() {
        let workspace = TempWorkspace::new().unwrap();

        let data = vec![1, 2, 3, 4, 5];
        workspace.write_bytes("test.bin", &data).unwrap();
        let read_data = workspace.read_bytes("test.bin").unwrap();

        assert_eq!(read_data, data);
    }

    #[test]
    fn test_exists() {
        let workspace = TempWorkspace::new().unwrap();

        assert!(!workspace.exists("test.txt"));

        workspace.write_file("test.txt", "content").unwrap();

        assert!(workspace.exists("test.txt"));
    }

    #[test]
    fn test_create_subdir() {
        let workspace = TempWorkspace::new().unwrap();

        let subdir = workspace.create_subdir("subdir").unwrap();

        assert!(subdir.exists());
        assert!(subdir.is_dir());
    }

    #[test]
    fn test_list_files() {
        let workspace = TempWorkspace::new().unwrap();

        workspace.write_file("file1.txt", "content1").unwrap();
        workspace
            .write_file("subdir/file2.txt", "content2")
            .unwrap();

        let files = workspace.list_files().unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("file1.txt")));
        assert!(files.iter().any(|p| p.ends_with("file2.txt")));
    }

    #[test]
    fn test_list_dirs() {
        let workspace = TempWorkspace::new().unwrap();

        workspace.create_subdir("dir1").unwrap();
        workspace.create_subdir("dir2/nested").unwrap();

        let dirs = workspace.list_dirs().unwrap();

        assert!(dirs.len() >= 2);
        assert!(dirs.iter().any(|p| p.ends_with("dir1")));
        assert!(dirs.iter().any(|p| p.ends_with("dir2")));
    }

    #[test]
    fn test_workspace_size() {
        let workspace = TempWorkspace::new().unwrap();

        workspace.write_file("file1.txt", "hello").unwrap();
        workspace.write_file("file2.txt", "world").unwrap();

        let size = workspace.size().unwrap();
        assert_eq!(size, 10); // "hello" + "world" = 10 bytes
    }

    #[test]
    fn test_file_count() {
        let workspace = TempWorkspace::new().unwrap();

        workspace.write_file("file1.txt", "a").unwrap();
        workspace.write_file("file2.txt", "b").unwrap();
        workspace.write_file("file3.txt", "c").unwrap();

        let count = workspace.file_count().unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_persist_workspace() {
        let path = {
            let workspace = TempWorkspace::new().unwrap();
            workspace.write_file("test.txt", "persist me").unwrap();
            workspace.persist().unwrap()
        };

        // Directory should still exist after persist
        assert!(path.exists());
        assert!(path.join("test.txt").exists());

        // Cleanup
        fs::remove_dir_all(&path).unwrap();
    }

    #[test]
    fn test_atomic_writer() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.txt");

        let writer = AtomicWriter::new(path.clone()).unwrap();
        writer.write("test content").unwrap();

        assert!(path.exists());
        assert_eq!(fs::read_to_string(&path).unwrap(), "test content");
    }

    #[test]
    fn test_atomic_writer_bytes() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("test.bin");

        let data = vec![1, 2, 3, 4, 5];
        let writer = AtomicWriter::new(path.clone()).unwrap();
        writer.write_bytes(&data).unwrap();

        assert!(path.exists());
        assert_eq!(fs::read(&path).unwrap(), data);
    }

    #[test]
    fn test_copy_dir_recursive() {
        let temp_src = TempDir::new().unwrap();
        let temp_dst = TempDir::new().unwrap();

        // Create nested structure
        fs::create_dir_all(temp_src.path().join("subdir")).unwrap();
        fs::write(temp_src.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_src.path().join("subdir/file2.txt"), "content2").unwrap();

        let dst = temp_dst.path().join("copied");
        copy_dir_recursive(temp_src.path(), &dst).unwrap();

        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("subdir/file2.txt").exists());
    }

    #[test]
    fn test_find_cargo_toml() {
        let temp = TempDir::new().unwrap();
        let cargo_toml = temp.path().join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"test\"").unwrap();

        let subdir = temp.path().join("src");
        fs::create_dir(&subdir).unwrap();

        let found = find_cargo_toml(&subdir);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), cargo_toml);
    }

    #[test]
    fn test_is_rust_project() {
        let temp = TempDir::new().unwrap();

        assert!(!is_rust_project(temp.path()));

        fs::write(temp.path().join("Cargo.toml"), "").unwrap();

        assert!(is_rust_project(temp.path()));
    }

    #[test]
    fn test_get_project_name() {
        let temp = TempDir::new().unwrap();
        let cargo_toml = temp.path().join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"my-project\"").unwrap();

        let name = get_project_name(temp.path()).unwrap();
        assert_eq!(name, "my-project");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_dir_size() {
        let temp = TempDir::new().unwrap();

        fs::write(temp.path().join("file1.txt"), "hello").unwrap();
        fs::write(temp.path().join("file2.txt"), "world").unwrap();

        let size = dir_size(temp.path()).unwrap();
        assert_eq!(size, 10);
    }

    #[test]
    fn test_remove_file_safe() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("test.txt");

        fs::write(&file, "content").unwrap();
        assert!(file.exists());

        remove_file_safe(&file).unwrap();
        assert!(!file.exists());

        // Should not error on non-existent file
        remove_file_safe(&file).unwrap();
    }

    #[test]
    fn test_remove_dir_safe() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("testdir");

        fs::create_dir(&dir).unwrap();
        fs::write(dir.join("file.txt"), "content").unwrap();
        assert!(dir.exists());

        remove_dir_safe(&dir).unwrap();
        assert!(!dir.exists());

        // Should not error on non-existent dir
        remove_dir_safe(&dir).unwrap();
    }

    #[test]
    fn test_copy_file_safe() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src.txt");
        let dst = temp.path().join("subdir/dst.txt");

        fs::write(&src, "content").unwrap();

        let bytes = copy_file_safe(&src, &dst).unwrap();

        assert_eq!(bytes, 7);
        assert!(dst.exists());
        assert_eq!(fs::read_to_string(&dst).unwrap(), "content");
    }

    #[test]
    fn test_copy_filtered() {
        let temp_src = TempDir::new().unwrap();

        fs::write(temp_src.path().join("file1.rs"), "rust").unwrap();
        fs::write(temp_src.path().join("file2.txt"), "text").unwrap();
        fs::write(temp_src.path().join("file3.rs"), "rust2").unwrap();

        let mut workspace = TempWorkspace::new().unwrap();
        let _dest = workspace
            .copy_filtered(temp_src.path(), &[".rs".to_string()])
            .unwrap();

        // Check files in workspace
        let files = workspace.list_files().unwrap();

        // Should have copied 2 .rs files
        let rs_count = files
            .iter()
            .filter(|p| p.to_string_lossy().ends_with(".rs"))
            .count();
        assert_eq!(rs_count, 2);

        // Should not have copied .txt file
        let txt_count = files
            .iter()
            .filter(|p| p.to_string_lossy().ends_with(".txt"))
            .count();
        assert_eq!(txt_count, 0);
    }

    #[test]
    fn test_original_path() {
        let temp_src = TempDir::new().unwrap();
        fs::write(temp_src.path().join("test.txt"), "content").unwrap();

        let mut workspace = TempWorkspace::new().unwrap();
        workspace.copy_project(temp_src.path()).unwrap();

        assert_eq!(workspace.original_path(), temp_src.path());
    }
}
