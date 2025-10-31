# Chapter 1: Project Foundation and Setup

## What This Chapter Accomplishes

Chapter 1 establishes the foundation for the entire BorrowScope project. It covers:

1. **Project Understanding** - What we're building and why
2. **Workspace Architecture** - How Rust workspaces organize multiple crates
3. **Initial Setup** - Creating the actual project structure
4. **Development Infrastructure** - Git, CI/CD, tooling, and documentation

## Why This Matters

Before writing any tracking or visualization code, we need:
- A properly structured workspace for our 3 main crates
- Version control to track changes
- Automated testing to catch bugs early
- Consistent tooling across all developers
- Clear documentation for users and contributors

## Implementation Strategy

### Sections with Code Implementation

**Section 03: Setting Up the Workspace**
- Creates the workspace root `Cargo.toml`
- Sets up three member crates:
  - `borrowscope-macro` - Procedural macros for code instrumentation
  - `borrowscope-runtime` - Runtime tracking system
  - `borrowscope-cli` - Command-line interface
- Configures shared dependencies
- Establishes project metadata

**Section 04: Git and Version Control**
- `.gitignore` - Excludes build artifacts and IDE files
- `.gitattributes` - Ensures consistent line endings

**Section 05: CI/CD Pipeline**
- GitHub Actions workflows for:
  - Automated testing on Linux, macOS, Windows
  - Code quality checks (clippy, rustfmt)
  - Release automation

**Section 06: Toolchain Configuration**
- `rust-toolchain.toml` - Pins Rust version
- `.rustfmt.toml` - Code formatting rules
- `.clippy.toml` - Linting configuration

**Section 07: Documentation Structure**
- `README.md` - Project overview
- `CONTRIBUTING.md` - Contribution guidelines
- License files

**Section 08: IDE Optimization**
- VS Code configuration for Rust development
- Debugging setup
- Recommended extensions

### Sections with Summary Documents

**Section 01: Understanding the Project Scope**
- Explains BorrowScope's purpose
- Describes the problem it solves
- Outlines core features

**Section 02: Rust Workspace Fundamentals**
- Teaches workspace concepts
- Explains package vs crate vs module
- Shows why workspaces are beneficial

## Learning Approach

As your Rust instructor, I'll explain:

### What
- What each file does
- What each configuration option means
- What problem each piece solves

### How
- How workspaces organize code
- How dependencies are shared
- How CI/CD automates quality checks

### Why
- Why we use workspaces instead of separate projects
- Why we configure tooling upfront
- Why documentation matters from day one

## Expected Outcomes

After completing Chapter 1, you'll have:

1. **A compilable workspace** with three empty crates
2. **Version control** properly configured
3. **Automated testing** via GitHub Actions
4. **Consistent tooling** across environments
5. **Documentation structure** ready for content
6. **Understanding** of Rust project organization

## Next Steps

Chapter 2 will begin implementing the procedural macro system that instruments user code to track ownership operations.

---

**Note:** This chapter is mostly setup and configuration. The real Rust programming begins in Chapter 2!
