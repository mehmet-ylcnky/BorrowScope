//! Workspace loading using ra_ap_* crates.

use std::path::Path;

use anyhow::Result;
use ra_ap_load_cargo::{load_workspace_at, LoadCargoConfig, ProcMacroServerChoice};
use ra_ap_project_model::{CargoConfig, RustLibSource};

/// Loaded workspace containing the semantic database and virtual file system.
pub struct WorkspaceData {
    pub db: ra_ap_ide_db::RootDatabase,
    pub vfs: ra_ap_vfs::Vfs,
}

/// Load a Rust workspace at the given path.
/// This is the expensive operation (~30-40s) that loads all dependencies and the sysroot.
pub fn load_workspace(root_path: &Path) -> Result<WorkspaceData> {
    let mut cargo_config = CargoConfig::default();
    cargo_config.sysroot = Some(RustLibSource::Discover);

    let load_config = LoadCargoConfig {
        load_out_dirs_from_check: true,
        with_proc_macro_server: ProcMacroServerChoice::None,
        prefill_caches: true,
        proc_macro_processes: 0,
    };

    let (db, vfs, _proc_macro_server) =
        load_workspace_at(root_path, &cargo_config, &load_config, &|msg| {
            tracing::debug!("Loading: {}", msg);
        })?;

    tracing::info!("Workspace loaded successfully");
    Ok(WorkspaceData { db, vfs })
}
