//! Global server state.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use lsp_types::InitializeParams;

use crate::workspace::WorkspaceData;

/// Content and status of an open file.
pub struct OpenFile {
    pub content: String,
    pub is_open: bool,
    pub dirty: bool,
}

pub struct GlobalState {
    /// Workspace data (None until loading completes)
    pub workspace: Option<WorkspaceData>,
    /// Whether shutdown has been requested
    pub shutdown_requested: bool,
    /// Workspace root path
    pub root_path: PathBuf,
    /// Open file contents (uri -> content)
    pub open_files: HashMap<String, OpenFile>,
}

impl GlobalState {
    pub fn new(params: &InitializeParams) -> Result<Self> {
        let root_path = params
            .root_uri
            .as_ref()
            .and_then(|uri| uri_to_path(uri))
            .or_else(|| params.root_path.as_ref().map(PathBuf::from))
            .ok_or_else(|| anyhow::anyhow!("No workspace root provided"))?;

        Ok(Self {
            workspace: None,
            shutdown_requested: false,
            root_path,
            open_files: HashMap::new(),
        })
    }

    pub fn is_ready(&self) -> bool {
        self.workspace.is_some()
    }

    /// Store or update file content.
    pub fn set_file_content(&mut self, uri: &str, content: String) {
        let entry = self.open_files.entry(uri.to_string()).or_insert(OpenFile {
            content: String::new(),
            is_open: true,
            dirty: false,
        });
        entry.content = content;
        entry.is_open = true;
        entry.dirty = true;
    }

    /// Mark a file as closed (but keep content for references).
    pub fn mark_file_closed(&mut self, uri: &str) {
        if let Some(file) = self.open_files.get_mut(uri) {
            file.is_open = false;
        }
    }

    /// Get file content if available.
    pub fn get_file_content(&self, uri: &str) -> Option<&str> {
        self.open_files.get(uri).map(|f| f.content.as_str())
    }

    /// Apply file changes to the Salsa database for incremental re-analysis.
    /// Returns the number of files updated in the VFS.
    pub fn apply_vfs_changes(&mut self) -> usize {
        let ws = match &mut self.workspace {
            Some(ws) => ws,
            None => return 0,
        };

        // Collect dirty file contents
        let dirty_files: Vec<(String, String)> = self
            .open_files
            .iter()
            .filter(|(_, f)| f.is_open && f.dirty)
            .filter_map(|(uri, f)| {
                uri.strip_prefix("file://")
                    .map(|path| (path.to_string(), f.content.clone()))
            })
            .collect();

        if dirty_files.is_empty() {
            return 0;
        }

        let count = dirty_files.len();

        // Push content into VFS
        for (path, content) in &dirty_files {
            let vfs_path = ra_ap_vfs::VfsPath::new_real_path(path.clone());
            ws.vfs
                .set_file_contents(vfs_path, Some(content.as_bytes().to_vec()));
        }

        // Apply VFS changes to the Salsa database
        let changes = ws.vfs.take_changes();
        if !changes.is_empty() {
            let mut change = ra_ap_ide_db::ChangeWithProcMacros::default();
            for (file_id, _) in &changes {
                // Find content for this file_id
                for (path, content) in &dirty_files {
                    let vfs_path = ra_ap_vfs::VfsPath::new_real_path(path.clone());
                    if let Some((fid, _)) = ws.vfs.file_id(&vfs_path) {
                        if fid == *file_id {
                            change
                                .source_change
                                .change_file(*file_id, Some(content.clone()));
                            break;
                        }
                    }
                }
            }
            ws.db.apply_change(change);
            tracing::debug!("Applied {} file changes to Salsa database", count);
        }

        // Mark all files as clean
        for file in self.open_files.values_mut() {
            file.dirty = false;
        }

        count
    }
}

fn uri_to_path(uri: &lsp_types::Uri) -> Option<PathBuf> {
    let s = uri.as_str();
    if let Some(path) = s.strip_prefix("file://") {
        Some(PathBuf::from(path))
    } else {
        None
    }
}
