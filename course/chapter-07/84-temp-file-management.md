# Section 84: Temporary File Management

## Learning Objectives

By the end of this section, you will:
- Implement safe temporary workspace management
- Handle file copying and cleanup
- Manage temporary build artifacts
- Implement atomic file operations
- Handle cleanup on errors and panics

## Prerequisites

- Section 83 (File Instrumentation)
- Understanding of RAII pattern
- Familiarity with file system operations

---

## Purpose

Temporary file management is critical for:
1. Isolating instrumented code from original source
2. Safe cleanup on errors or interruptions
3. Preventing conflicts with existing builds
4. Managing multiple concurrent runs
5. Atomic operations for reliability

---

## Complete Implementation

**src/utils.rs:**

```rust
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use tempfile::TempDir;
use walkdir::WalkDir;

/// Manages a temporary workspace for instrumented code
pub struct TempWorkspace {
    dir: TempDir,
    original_path: PathBuf,
}

impl TempWorkspace {
    /// Create a new temporary workspace
    pub fn new() -> Result<Self> {
        let dir = TempDir::new()
            .context("Failed to create temporary directory")?;
        
        Ok(Self {
            dir,
            original_path: PathBuf::new(),
        })
    }
    
    /// Create with a specific prefix
    pub fn with_prefix(prefix: &str) -> Result<Self> {
        let dir = TempDir::with_prefix(prefix)
            .context("Failed to create temporary directory")?;
        
        Ok(Self {
            dir,
            original_path: PathBuf::new(),
        })
    }
    
    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        self.dir.path()
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
            .filter_entry(|e| should_include(e.path(), patterns))
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() {
                let relative = path.strip_prefix(source)?;
                let dest_path = dest.join(relative);
                
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                fs::copy(path, dest_path)?;
            }
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
    
    /// Read a file from the workspace
    pub fn read_file(&self, relative_path: &str) -> Result<String> {
        let path = self.dir.path().join(relative_path);
        fs::read_to_string(&path)
            .context("Failed to read file from workspace")
    }
    
    /// List all files in the workspace
    pub fn list_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        for entry in WalkDir::new(self.dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                files.push(entry.path().to_path_buf());
            }
        }
        
        Ok(files)
    }
    
    /// Get the size of the workspace in bytes
    pub fn size(&self) -> Result<u64> {
        let mut total = 0;
        
        for entry in WalkDir::new(self.dir.path())
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                total += entry.metadata()?.len();
            }
        }
        
        Ok(total)
    }
    
    /// Explicitly cleanup the workspace
    /// (Note: cleanup happens automatically on drop)
    pub fn cleanup(self) -> Result<()> {
        // TempDir handles cleanup on drop
        Ok(())
    }
    
    /// Keep the temporary directory (don't delete on drop)
    pub fn persist(self) -> Result<PathBuf> {
        let path = self.dir.path().to_path_buf();
        let _ = self.dir.into_path();
        Ok(path)
    }
}

/// Copy a directory recursively
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
            std::os::unix::fs::symlink(&target, &dst_path)?;
            #[cfg(windows)]
            {
                if target.is_dir() {
                    std::os::windows::fs::symlink_dir(&target, &dst_path)?;
                } else {
                    std::os::windows::fs::symlink_file(&target, &dst_path)?;
                }
            }
        }
    }
    
    Ok(())
}

fn should_include(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    
    // Skip hidden files and directories
    if path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
    {
        return false;
    }
    
    // Check patterns
    for pattern in patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }
    
    // Include by default if no patterns specified
    patterns.is_empty()
}

/// Atomic file writer - writes to temp file then renames
pub struct AtomicWriter {
    temp_path: PathBuf,
    final_path: PathBuf,
}

impl AtomicWriter {
    pub fn new(path: PathBuf) -> Result<Self> {
        let temp_path = path.with_extension("tmp");
        
        Ok(Self {
            temp_path,
            final_path: path,
        })
    }
    
    pub fn write(&self, content: &str) -> Result<()> {
        // Write to temporary file
        fs::write(&self.temp_path, content)
            .context("Failed to write to temporary file")?;
        
        // Atomically rename to final path
        fs::rename(&self.temp_path, &self.final_path)
            .context("Failed to rename temporary file")?;
        
        Ok(())
    }
    
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
pub fn is_rust_project(path: &Path) -> bool {
    path.join("Cargo.toml").exists()
}

/// Get project name from Cargo.toml
pub fn get_project_name(path: &Path) -> Result<String> {
    let cargo_toml = find_cargo_toml(path)
        .ok_or_else(|| anyhow::anyhow!("No Cargo.toml found"))?;
    
    let content = fs::read_to_string(cargo_toml)?;
    let toml: toml::Value = toml::from_str(&content)?;
    
    let name = toml.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .ok_or_else(|| anyhow::anyhow!("No package name in Cargo.toml"))?;
    
    Ok(name.to_string())
}

/// Calculate directory size
pub fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0;
    
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().is_file() {
            total += entry.metadata()?.len();
        }
    }
    
    Ok(total)
}

/// Format bytes as human-readable string
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
pub fn remove_file_safe(path: &Path) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_MS: u64 = 100;
    
    for attempt in 0..MAX_RETRIES {
        match fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) if attempt < MAX_RETRIES - 1 => {
                std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
    
    Ok(())
}

/// Safe directory removal with retry
pub fn remove_dir_safe(path: &Path) -> Result<()> {
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY_MS: u64 = 100;
    
    for attempt in 0..MAX_RETRIES {
        match fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) if attempt < MAX_RETRIES - 1 => {
                std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                continue;
            }
            Err(e) => return Err(e.into()),
        }
    }
    
    Ok(())
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
    fn test_temp_workspace_cleanup() {
        let path = {
            let workspace = TempWorkspace::new().unwrap();
            workspace.path().to_path_buf()
        };
        
        // After drop, directory should be cleaned up
        assert!(!path.exists());
    }

    #[test]
    fn test_copy_project() {
        let temp_src = TempDir::new().unwrap();
        let src_file = temp_src.path().join("test.txt");
        fs::write(&src_file, "test content").unwrap();
        
        let mut workspace = TempWorkspace::new().unwrap();
        let dest = workspace.copy_project(temp_src.path()).unwrap();
        
        assert!(dest.exists());
        assert!(dest.join("test.txt").exists());
    }

    #[test]
    fn test_write_and_read_file() {
        let workspace = TempWorkspace::new().unwrap();
        
        workspace.write_file("test.txt", "hello world").unwrap();
        let content = workspace.read_file("test.txt").unwrap();
        
        assert_eq!(content, "hello world");
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
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 bytes");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
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
}
```

---

## Usage Examples

```rust
use borrowscope_cli::utils::TempWorkspace;

// Create temporary workspace
let mut workspace = TempWorkspace::new()?;

// Copy project
let project_copy = workspace.copy_project(Path::new("./my_project"))?;

// Work with copied files
workspace.write_file("instrumented/main.rs", instrumented_code)?;

// Read results
let output = workspace.read_file("output.json")?;

// Cleanup happens automatically on drop
```

---

## Advanced Usage

### Persistent Workspace

```rust
// Keep workspace after function returns
let workspace = TempWorkspace::new()?;
workspace.write_file("important.txt", "data")?;

let path = workspace.persist()?;
println!("Workspace saved at: {:?}", path);

// Manual cleanup later
std::fs::remove_dir_all(&path)?;
```

### Atomic Writes

```rust
use borrowscope_cli::utils::AtomicWriter;

let writer = AtomicWriter::new(PathBuf::from("output.json"))?;
writer.write(&json_data)?;

// File is written atomically - either complete or not at all
```

---

## Key Takeaways

✅ **RAII pattern** - Automatic cleanup on drop  
✅ **Atomic operations** - Safe file writes  
✅ **Recursive copying** - Handle complex directory structures  
✅ **Error handling** - Retry logic for flaky operations  
✅ **Cross-platform** - Works on Linux, macOS, Windows  

---

**Previous:** [83-file-instrumentation.md](./83-file-instrumentation.md)  
**Next:** [85-cargo-integration.md](./85-cargo-integration.md)

**Progress:** 9/13 ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬜⬜⬜⬜
