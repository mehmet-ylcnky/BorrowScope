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

/// Cached analysis result for a function.
#[derive(Debug, Clone)]
pub enum AnalysisState {
    /// Fresh result from latest analysis.
    Ready(serde_json::Value),
    /// Previous result, file has changed since.
    Stale(serde_json::Value),
}

/// Per-file analysis cache.
#[derive(Debug, Default)]
pub struct AnalysisCache {
    /// function_name -> cached result
    pub functions: HashMap<String, AnalysisState>,
}

impl AnalysisCache {
    /// Get cached result (fresh or stale).
    pub fn get(&self, function_name: &str) -> Option<&serde_json::Value> {
        match self.functions.get(function_name) {
            Some(AnalysisState::Ready(v)) | Some(AnalysisState::Stale(v)) => Some(v),
            None => None,
        }
    }

    /// Check if result is stale.
    pub fn is_stale(&self, function_name: &str) -> bool {
        matches!(self.functions.get(function_name), Some(AnalysisState::Stale(_)))
    }

    /// Store a fresh result.
    pub fn set_ready(&mut self, function_name: String, value: serde_json::Value) {
        self.functions.insert(function_name, AnalysisState::Ready(value));
    }

    /// Mark all entries as stale.
    pub fn mark_all_stale(&mut self) {
        for state in self.functions.values_mut() {
            if let AnalysisState::Ready(v) = state.clone() {
                *state = AnalysisState::Stale(v);
            }
        }
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.functions.clear();
    }
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
    /// Receiver for background workspace loading result
    pub loading_receiver: Option<crossbeam_channel::Receiver<anyhow::Result<WorkspaceData>>>,
    /// Debounce: pending files that changed (uri -> previous content)
    pub pending_changes: Vec<(String, Option<String>)>,
    /// Debounce: timestamp of last change
    pub last_change_time: Option<std::time::Instant>,
    /// Debounce duration in milliseconds
    pub debounce_ms: u64,
    /// Per-file analysis cache (uri -> cache)
    pub analysis_cache: HashMap<String, AnalysisCache>,
}

impl GlobalState {
    pub fn new(params: &InitializeParams) -> Result<Self> {
        let root_path = params
            .root_uri
            .as_ref()
            .and_then(|uri| uri_to_path(uri))
            .or_else(|| params.root_path.as_ref().map(PathBuf::from))
            .ok_or_else(|| anyhow::anyhow!("No workspace root provided"))?;

        // Read debounce from initialization options
        let debounce_ms = params
            .initialization_options
            .as_ref()
            .and_then(|opts| opts.get("debounceMs"))
            .and_then(|v| v.as_u64())
            .unwrap_or(300);

        Ok(Self {
            workspace: None,
            shutdown_requested: false,
            root_path,
            open_files: HashMap::new(),
            loading_receiver: None,
            pending_changes: Vec::new(),
            last_change_time: None,
            debounce_ms,
            analysis_cache: HashMap::new(),
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
    pub fn apply_vfs_changes(&mut self) -> Vec<String> {
        let ws = match &mut self.workspace {
            Some(ws) => ws,
            None => return vec![],
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
            return vec![];
        }

        let modified_paths: Vec<String> = dirty_files.iter().map(|(p, _)| p.clone()).collect();

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
        }

        // Mark all files as clean
        for file in self.open_files.values_mut() {
            file.dirty = false;
        }

        modified_paths
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
