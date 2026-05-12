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
        });
        entry.content = content;
        entry.is_open = true;
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
}

fn uri_to_path(uri: &lsp_types::Uri) -> Option<PathBuf> {
    let s = uri.as_str();
    if let Some(path) = s.strip_prefix("file://") {
        Some(PathBuf::from(path))
    } else {
        None
    }
}
