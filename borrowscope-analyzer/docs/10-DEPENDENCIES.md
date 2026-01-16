## 10. Dependencies

The borrowscope-analyzer relies on rust-analyzer's published crate ecosystem for semantic analysis. These crates are versioned together and should be kept in sync to avoid compatibility issues.

### rust-analyzer Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| `ra_ap_hir` | 0.0.232 | High-level intermediate representation and semantic queries |
| `ra_ap_ide_db` | 0.0.232 | IDE database infrastructure and root database type |
| `ra_ap_load-cargo` | 0.0.232 | Cargo workspace loading and sysroot discovery |
| `ra_ap_project_model` | 0.0.232 | Project structure modeling and configuration |
| `ra_ap_syntax` | 0.0.232 | Syntax tree representation and AST types |
| `ra_ap_vfs` | 0.0.232 | Virtual file system for source file management |

These crates are published to crates.io with each rust-analyzer release. The version number `0.0.232` corresponds to a specific rust-analyzer release. All `ra_ap_*` crates must use the same version to ensure ABI compatibility.

**Version Pinning Rationale**: The `ra_ap_*` crates follow rust-analyzer's rapid release cycle and do not maintain semver compatibility between versions. Internal APIs change frequently as rust-analyzer evolves. Pinning to a specific version ensures reproducible builds and avoids unexpected breakage from API changes.

**Updating Dependencies**: When updating to a newer rust-analyzer version, all `ra_ap_*` crates must be updated together. API changes may require code modifications. The rust-analyzer changelog documents breaking changes between versions.

### Utility Crates

| Crate | Version | Purpose |
|-------|---------|---------|
| `serde` | 1.0 | Serialization framework |
| `serde_json` | 1.0 | JSON serialization for output |
| `anyhow` | 1.0 | Error handling with context |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log output formatting |

These utility crates follow semver and can be updated independently.

### Rust Version Requirements

The analyzer requires Rust 1.75 or later due to dependencies on recent language features used by rust-analyzer. The `rust-version` field in `Cargo.toml` enforces this minimum.

### Dependency Tree Considerations

The `ra_ap_*` crates bring a substantial dependency tree, including:

- `salsa` - Incremental computation framework
- `rowan` - Syntax tree library
- `chalk` - Trait solving
- `rustc_lexer` - Rust lexer (from rustc)
- Numerous utility crates

This results in a large dependency footprint (~300 crates) and extended initial compile times (~3-5 minutes for a clean build). Subsequent incremental builds are fast (~5-10 seconds).

### Compatibility Notes

The `ra_ap_*` crates are designed for use within rust-analyzer and may have rough edges when used as a library:

- Some APIs assume IDE usage patterns and may be awkward for batch processing
- Error handling sometimes uses panics rather than Results
- Documentation is sparse; reading rust-analyzer source code is often necessary
- Breaking changes occur without deprecation warnings

Despite these challenges, the `ra_ap_*` crates provide the most complete and correct Rust semantic analysis available outside of rustc itself. The alternative—reimplementing type resolution—would be a multi-year effort with ongoing maintenance burden as Rust evolves.
