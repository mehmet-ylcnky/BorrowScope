# Section 208: Open Source Licensing

## Learning Objectives

By the end of this section, you will:
- Master open source licensing for BorrowScope
- Implement production deployment strategies
- Ensure smooth user experience
- Build sustainable project infrastructure
- Maintain long-term project health

## Prerequisites

- Completed Chapters 1-13
- Understanding of deployment processes
- Familiarity with packaging tools
- Knowledge of distribution platforms
- Experience with project management

## Introduction

Open Source Licensing is the final step in bringing BorrowScope to users. This section covers everything needed to package, distribute, and maintain BorrowScope as a production-ready application.

Successful deployment requires careful attention to packaging, distribution, user experience, and long-term maintenance. This section provides comprehensive guidance for launching and sustaining BorrowScope.

## Core Concepts

### Deployment Fundamentals

Key deployment principles:

1. **Reliability**: Consistent installation experience
2. **Accessibility**: Easy to discover and install
3. **Updates**: Seamless update mechanism
4. **Support**: Clear documentation and help
5. **Sustainability**: Long-term maintenance plan

### Architecture

```rust
/// Deployment manager
pub struct DeploymentManager {
    config: DeploymentConfig,
    packager: Packager,
    distributor: Distributor,
}

impl DeploymentManager {
    pub fn new(config: DeploymentConfig) -> Self {
        Self {
            config,
            packager: Packager::new(),
            distributor: Distributor::new(),
        }
    }
    
    pub fn package(&self, target: Target) -> Result<Package> {
        println!("Packaging for {:?}", target);
        
        let artifacts = self.build_artifacts(target)?;
        let package = self.packager.create_package(artifacts, target)?;
        
        self.validate_package(&package)?;
        
        Ok(package)
    }
    
    fn build_artifacts(&self, target: Target) -> Result<Artifacts> {
        let mut artifacts = Artifacts::new();
        
        // Build binary
        let binary = self.build_binary(target)?;
        artifacts.add_binary(binary);
        
        // Add resources
        artifacts.add_resources(self.collect_resources()?);
        
        // Add documentation
        artifacts.add_docs(self.generate_docs()?);
        
        Ok(artifacts)
    }
    
    fn build_binary(&self, target: Target) -> Result<Binary> {
        let output = std::process::Command::new("cargo")
            .args(&["build", "--release", "--target", &target.triple()])
            .output()?;
        
        if !output.status.success() {
            return Err(Error::BuildFailed);
        }
        
        Ok(Binary {
            path: format!("target/{}/release/borrowscope", target.triple()),
            target,
        })
    }
    
    fn collect_resources(&self) -> Result<Vec<Resource>> {
        let mut resources = Vec::new();
        
        // Icons
        resources.push(Resource {
            path: "assets/icon.png".into(),
            resource_type: ResourceType::Icon,
        });
        
        // Config files
        resources.push(Resource {
            path: "config/default.toml".into(),
            resource_type: ResourceType::Config,
        });
        
        Ok(resources)
    }
    
    fn generate_docs(&self) -> Result<Vec<Document>> {
        let mut docs = Vec::new();
        
        docs.push(Document {
            path: "README.md".into(),
            content: self.generate_readme()?,
        });
        
        docs.push(Document {
            path: "LICENSE".into(),
            content: self.load_license()?,
        });
        
        Ok(docs)
    }
    
    fn generate_readme(&self) -> Result<String> {
        Ok(format!(
            "# BorrowScope\n\nVersion: {}\n\n## Installation\n\n...",
            env!("CARGO_PKG_VERSION")
        ))
    }
    
    fn load_license(&self) -> Result<String> {
        std::fs::read_to_string("LICENSE")
            .map_err(|e| Error::IoError(e))
    }
    
    fn validate_package(&self, package: &Package) -> Result<()> {
        // Verify package integrity
        if package.artifacts.is_empty() {
            return Err(Error::InvalidPackage("No artifacts".into()));
        }
        
        // Check signatures
        if self.config.sign_packages {
            self.verify_signature(package)?;
        }
        
        Ok(())
    }
    
    fn verify_signature(&self, package: &Package) -> Result<()> {
        // Signature verification logic
        Ok(())
    }
    
    pub fn distribute(&self, package: Package) -> Result<()> {
        self.distributor.upload(package, &self.config.distribution_channels)?;
        Ok(())
    }
}

/// Deployment configuration
pub struct DeploymentConfig {
    pub sign_packages: bool,
    pub distribution_channels: Vec<Channel>,
    pub auto_update_enabled: bool,
}

/// Target platform
#[derive(Debug, Clone, Copy)]
pub enum Target {
    LinuxX64,
    LinuxArm64,
    MacOSX64,
    MacOSArm64,
    WindowsX64,
}

impl Target {
    pub fn triple(&self) -> &str {
        match self {
            Target::LinuxX64 => "x86_64-unknown-linux-gnu",
            Target::LinuxArm64 => "aarch64-unknown-linux-gnu",
            Target::MacOSX64 => "x86_64-apple-darwin",
            Target::MacOSArm64 => "aarch64-apple-darwin",
            Target::WindowsX64 => "x86_64-pc-windows-msvc",
        }
    }
}

/// Package
pub struct Package {
    pub artifacts: Vec<Artifact>,
    pub metadata: PackageMetadata,
}

/// Artifacts
pub struct Artifacts {
    binaries: Vec<Binary>,
    resources: Vec<Resource>,
    docs: Vec<Document>,
}

impl Artifacts {
    pub fn new() -> Self {
        Self {
            binaries: Vec::new(),
            resources: Vec::new(),
            docs: Vec::new(),
        }
    }
    
    pub fn add_binary(&mut self, binary: Binary) {
        self.binaries.push(binary);
    }
    
    pub fn add_resources(&mut self, resources: Vec<Resource>) {
        self.resources.extend(resources);
    }
    
    pub fn add_docs(&mut self, docs: Vec<Document>) {
        self.docs.extend(docs);
    }
    
    pub fn is_empty(&self) -> bool {
        self.binaries.is_empty() && self.resources.is_empty() && self.docs.is_empty()
    }
}

pub struct Binary {
    pub path: String,
    pub target: Target,
}

pub struct Resource {
    pub path: PathBuf,
    pub resource_type: ResourceType,
}

pub enum ResourceType {
    Icon,
    Config,
    Asset,
}

pub struct Document {
    pub path: PathBuf,
    pub content: String,
}

pub struct PackageMetadata {
    pub version: String,
    pub name: String,
    pub description: String,
}

pub enum Artifact {
    Binary(Binary),
    Resource(Resource),
    Document(Document),
}

/// Packager
pub struct Packager;

impl Packager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn create_package(&self, artifacts: Artifacts, target: Target) -> Result<Package> {
        let package_artifacts = self.convert_artifacts(artifacts);
        
        Ok(Package {
            artifacts: package_artifacts,
            metadata: PackageMetadata {
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "BorrowScope".to_string(),
                description: "Rust ownership visualizer".to_string(),
            },
        })
    }
    
    fn convert_artifacts(&self, artifacts: Artifacts) -> Vec<Artifact> {
        let mut result = Vec::new();
        
        for binary in artifacts.binaries {
            result.push(Artifact::Binary(binary));
        }
        
        for resource in artifacts.resources {
            result.push(Artifact::Resource(resource));
        }
        
        for doc in artifacts.docs {
            result.push(Artifact::Document(doc));
        }
        
        result
    }
}

/// Distributor
pub struct Distributor;

impl Distributor {
    pub fn new() -> Self {
        Self
    }
    
    pub fn upload(&self, package: Package, channels: &[Channel]) -> Result<()> {
        for channel in channels {
            self.upload_to_channel(&package, channel)?;
        }
        Ok(())
    }
    
    fn upload_to_channel(&self, package: &Package, channel: &Channel) -> Result<()> {
        println!("Uploading to {:?}", channel);
        // Upload logic
        Ok(())
    }
}

pub enum Channel {
    GitHub,
    CratesIo,
    Homebrew,
    Chocolatey,
    Snap,
    Flatpak,
}

#[derive(Debug)]
pub enum Error {
    BuildFailed,
    InvalidPackage(String),
    IoError(std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;
```

## Implementation

### Platform-Specific Packaging

```rust
/// Platform packager
pub trait PlatformPackager {
    fn package(&self, artifacts: &Artifacts) -> Result<Vec<u8>>;
    fn installer_format(&self) -> &str;
}

/// Linux packager (DEB/RPM)
pub struct LinuxPackager;

impl PlatformPackager for LinuxPackager {
    fn package(&self, artifacts: &Artifacts) -> Result<Vec<u8>> {
        // Create DEB package
        let mut package = Vec::new();
        
        // Add control file
        // Add binaries
        // Add desktop entry
        
        Ok(package)
    }
    
    fn installer_format(&self) -> &str {
        "deb"
    }
}

/// macOS packager (DMG/PKG)
pub struct MacOSPackager;

impl PlatformPackager for MacOSPackager {
    fn package(&self, artifacts: &Artifacts) -> Result<Vec<u8>> {
        // Create app bundle
        // Sign with codesign
        // Create DMG
        
        Ok(Vec::new())
    }
    
    fn installer_format(&self) -> &str {
        "dmg"
    }
}

/// Windows packager (MSI/EXE)
pub struct WindowsPackager;

impl PlatformPackager for WindowsPackager {
    fn package(&self, artifacts: &Artifacts) -> Result<Vec<u8>> {
        // Create MSI installer
        // Sign with signtool
        
        Ok(Vec::new())
    }
    
    fn installer_format(&self) -> &str {
        "msi"
    }
}
```

### Auto-Update System

```rust
/// Auto-update manager
pub struct AutoUpdateManager {
    current_version: Version,
    update_url: String,
}

impl AutoUpdateManager {
    pub fn new(current_version: Version, update_url: String) -> Self {
        Self {
            current_version,
            update_url,
        }
    }
    
    pub async fn check_for_updates(&self) -> Result<Option<Update>> {
        let latest = self.fetch_latest_version().await?;
        
        if latest > self.current_version {
            Ok(Some(Update {
                version: latest,
                download_url: format!("{}/v{}", self.update_url, latest),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn fetch_latest_version(&self) -> Result<Version> {
        // Fetch from update server
        Ok(Version::new(0, 1, 0))
    }
    
    pub async fn download_and_install(&self, update: Update) -> Result<()> {
        // Download update
        let package = self.download_update(&update).await?;
        
        // Verify signature
        self.verify_update(&package)?;
        
        // Install update
        self.install_update(package)?;
        
        Ok(())
    }
    
    async fn download_update(&self, update: &Update) -> Result<Vec<u8>> {
        // Download logic
        Ok(Vec::new())
    }
    
    fn verify_update(&self, package: &[u8]) -> Result<()> {
        // Verify signature
        Ok(())
    }
    
    fn install_update(&self, package: Vec<u8>) -> Result<()> {
        // Install logic
        Ok(())
    }
}

pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.major.cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Eq for Version {}

pub struct Update {
    pub version: Version,
    pub download_url: String,
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_deployment_manager() {
        let config = DeploymentConfig {
            sign_packages: false,
            distribution_channels: vec![],
            auto_update_enabled: true,
        };
        
        let manager = DeploymentManager::new(config);
        assert!(true);
    }
    
    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(1, 1, 0);
        
        assert!(v2 > v1);
    }
    
    #[test]
    fn test_target_triple() {
        let target = Target::LinuxX64;
        assert_eq!(target.triple(), "x86_64-unknown-linux-gnu");
    }
}
```

## Best Practices

1. **Automation**: Automate packaging and distribution
2. **Testing**: Test installers on clean systems
3. **Documentation**: Provide clear installation instructions
4. **Updates**: Implement seamless update mechanism
5. **Support**: Offer multiple support channels

## Common Pitfalls

1. **Platform Differences**: Test on all target platforms
2. **Dependencies**: Bundle or document all dependencies
3. **Permissions**: Handle permission requirements properly
4. **Updates**: Test update process thoroughly
5. **Rollback**: Provide rollback mechanism for failed updates

## Key Takeaways

- Open Source Licensing is critical for user adoption
- Automation reduces deployment errors
- Multi-platform support requires careful testing
- Updates should be seamless and reliable
- Documentation and support are essential

## Further Reading

- [Rust Packaging Guide](https://doc.rust-lang.org/cargo/guide/)
- [Cross-Platform Deployment](https://tauri.app/v1/guides/building/)
- [Software Distribution](https://en.wikipedia.org/wiki/Software_distribution)
- [Semantic Versioning](https://semver.org/)

## Summary

This section covered open source licensing with comprehensive examples and best practices for successfully deploying and maintaining BorrowScope in production.
