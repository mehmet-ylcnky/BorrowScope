//! Global server state.

use std::path::PathBuf;

use anyhow::Result;
use lsp_types::InitializeParams;

use crate::workspace::WorkspaceData;

pub struct GlobalState {
    /// Workspace data (None until loading completes)
    pub workspace: Option<WorkspaceData>,
    /// Whether shutdown has been requested
    pub shutdown_requested: bool,
    /// Workspace root path
    pub root_path: PathBuf,
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
        })
    }

    pub fn is_ready(&self) -> bool {
        self.workspace.is_some()
    }
}

/// Convert an LSP URI to a filesystem path.
fn uri_to_path(uri: &lsp_types::Uri) -> Option<PathBuf> {
    let s = uri.as_str();
    if let Some(path) = s.strip_prefix("file://") {
        Some(PathBuf::from(path))
    } else {
        None
    }
}
