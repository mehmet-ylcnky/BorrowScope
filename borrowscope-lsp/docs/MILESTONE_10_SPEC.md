# Milestone 10: Publishing and Distribution - Detailed Specification

## 10.1 VS Code Marketplace Listing

**Objective:** Create a compelling marketplace listing that communicates what BorrowScope does, who it's for, and what it looks like. The listing should rank well for searches like "rust ownership", "borrow checker visualization", and "rust learning tool."

**Steps:**
1. Create extension icon (128x128 PNG, recognizable at small sizes)
2. Write marketplace description (short + detailed)
3. Capture screenshots showing key features (graph, decorations, timeline)
4. Record a 30-second demo GIF showing the live update experience
5. Add categories, tags, and keywords for discoverability
6. Set up publisher account on VS Code Marketplace

**package.json marketplace fields:**
```json
{
  "publisher": "borrowscope",
  "icon": "media/icon.png",
  "galleryBanner": { "color": "#1e1e1e", "theme": "dark" },
  "categories": ["Programming Languages", "Visualization", "Education", "Linters"],
  "keywords": ["rust", "ownership", "borrow checker", "visualization", "memory safety"],
  "badges": [
    { "url": "https://img.shields.io/github/stars/mehmet-ylcnky/BorrowScope", "href": "https://github.com/mehmet-ylcnky/BorrowScope" }
  ],
  "repository": { "type": "git", "url": "https://github.com/mehmet-ylcnky/BorrowScope" },
  "homepage": "https://mehmet-ylcnky.github.io/BorrowScope/"
}
```

**README.md structure for marketplace:**
```markdown
# BorrowScope - Real-Time Ownership Visualization for Rust

> See Rust's ownership system as it happens. No build step. No configuration.

![Demo GIF](media/demo.gif)

## Features

- **Live Ownership Graph** — Interactive force-directed graph showing variable relationships
- **Borrow Scope Highlighting** — Colored regions showing exactly where borrows are active
- **Conflict Detection** — Educational diagnostics explaining WHY the borrow checker rejects patterns
- **Inline Annotations** — [Rc], [&mut], [&] hints next to variable declarations
- **Timeline View** — Gantt-chart showing variable lifetimes and borrow overlaps
- **Move Chain Tracking** — Trace ownership transfers through your code

## Quick Start

1. Install the extension
2. Open any Rust project
3. Wait for "BorrowScope: Ready" in the status bar (~30s first time)
4. See ownership annotations appear automatically

## Screenshots

[Graph Panel] [Borrow Scopes] [Timeline] [Conflict Detection]

## Requirements

- VS Code 1.85+
- Rust toolchain (rustc, cargo)
- A Cargo-based Rust project

## How It Works

BorrowScope runs its own semantic analysis engine (powered by the same ra_ap_* crates
as rust-analyzer) to understand your code's ownership structure. It operates as a
companion language server alongside rust-analyzer, providing ownership-specific
visualizations that RA doesn't offer.
```

**Screenshots to capture:**
1. Graph panel showing a function with Rc, borrows, and moves (hero image)
2. Borrow scope highlighting (blue/red backgrounds in editor)
3. Conflict diagnostic in Problems panel with related locations
4. Timeline view with overlapping borrow scopes
5. CodeLens showing "8 vars, 3 borrows, 1 move" above a function
6. Dark theme + light theme side by side

**Expectation:** The listing is professional, informative, and visually compelling. A Rust developer browsing the marketplace understands the value proposition within 5 seconds.

**Tests for 10.1:**
- Icon renders clearly at 32x32 (marketplace search results)
- README renders correctly on marketplace (no broken images)
- All screenshots are current (match actual extension behavior)
- Demo GIF is under 5MB (marketplace limit)
- Keywords match common search terms (verify with marketplace search)
- Links (repository, homepage) are valid

---

## 10.2 Extension Bundling

**Objective:** Bundle the TypeScript extension code into a single minified file using esbuild. This reduces the extension's install size and startup time. The server binary is NOT bundled here (handled in 10.3).

**Steps:**
1. Configure esbuild to bundle `src/extension.ts` into `out/extension.js`
2. Externalize `vscode` module (provided by VS Code at runtime)
3. Include D3.js and other WebView assets in `media/`
4. Generate source maps for debugging
5. Verify bundled extension activates correctly

**esbuild.js:**
```javascript
const esbuild = require('esbuild');

const production = process.argv.includes('--production');

esbuild.build({
    entryPoints: ['src/extension.ts'],
    bundle: true,
    outfile: 'out/extension.js',
    external: ['vscode'],
    format: 'cjs',
    platform: 'node',
    target: 'node18',
    sourcemap: !production,
    minify: production,
    treeShaking: true,
}).catch(() => process.exit(1));
```

**.vscodeignore (exclude from package):**
```
src/**
node_modules/**
.vscode/**
tsconfig.json
esbuild.js
*.ts
!out/**
!media/**
!server/**
```

**Size targets:**
```
Component              │ Unbundled  │ Bundled
───────────────────────┼────────────┼─────────
extension.js           │ ~500KB     │ ~80KB (minified)
node_modules           │ ~50MB      │ 0 (bundled in)
media/ (D3, CSS, HTML) │ ~300KB     │ ~300KB (unchanged)
server binary          │ ~30MB      │ ~30MB (per platform)
───────────────────────┼────────────┼─────────
Total package          │ ~80MB      │ ~31MB
```

**Expectation:** The packaged `.vsix` file is under 35MB (single platform) or under 100MB (all platforms bundled). Extension activates in < 500ms.

**Tests for 10.2:**
- `esbuild` produces `out/extension.js` without errors
- Bundled extension activates in VS Code
- No missing modules at runtime (all dependencies bundled)
- Source maps work for debugging (breakpoints hit)
- Production build is minified (no readable variable names)
- Package size is under 35MB (single platform)

---

## 10.3 Platform-Specific Binaries

**Objective:** Build and distribute the `borrowscope-lsp` binary for all major platforms. The extension downloads the correct binary on first activation or bundles it in platform-specific packages.

**Target platforms:**
```
Platform          │ Target Triple                │ Binary Name
──────────────────┼──────────────────────────────┼─────────────────────
Linux x64         │ x86_64-unknown-linux-gnu     │ borrowscope-lsp
Linux ARM64       │ aarch64-unknown-linux-gnu    │ borrowscope-lsp
macOS x64         │ x86_64-apple-darwin          │ borrowscope-lsp
macOS ARM64 (M1+) │ aarch64-apple-darwin         │ borrowscope-lsp
Windows x64       │ x86_64-pc-windows-msvc       │ borrowscope-lsp.exe
```

**Distribution strategy:**

**Option A: Download on first use**
- Extension ships without binary (small .vsix, ~1MB)
- On first activation, downloads correct binary from GitHub Releases
- Shows progress notification during download
- Caches in `globalStoragePath`

**Option B: Platform-specific .vsix packages**
- Build separate .vsix for each platform (like rust-analyzer does)
- Marketplace supports platform-specific packages since VS Code 1.61
- No download step; binary is included
- Larger package size (~35MB per platform)

**Recommended: Option B** (same approach as rust-analyzer extension)

**package.json for platform packages:**
```json
{
  "engines": { "vscode": "^1.85.0" },
  "os": ["linux"],
  "cpu": ["x64"],
  "files": ["out/**", "media/**", "server/borrowscope-lsp"]
}
```

**GitHub Actions build matrix:**
```yaml
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
        artifact: borrowscope-lsp
      - os: ubuntu-latest
        target: aarch64-unknown-linux-gnu
        artifact: borrowscope-lsp
      - os: macos-latest
        target: x86_64-apple-darwin
        artifact: borrowscope-lsp
      - os: macos-latest
        target: aarch64-apple-darwin
        artifact: borrowscope-lsp
      - os: windows-latest
        target: x86_64-pc-windows-msvc
        artifact: borrowscope-lsp.exe

steps:
  - uses: actions/checkout@v4
  - uses: dtolnay/rust-toolchain@stable
    with:
      targets: ${{ matrix.target }}
  - run: cargo build --release -p borrowscope-lsp --target ${{ matrix.target }}
  - uses: actions/upload-artifact@v4
    with:
      name: server-${{ matrix.target }}
      path: target/${{ matrix.target }}/release/${{ matrix.artifact }}
```

**Expectation:** Users on any platform install the extension and it works immediately. No manual binary installation, no PATH configuration, no Rust toolchain needed for the server itself.

**Tests for 10.3:**
- Binary runs on each target platform (CI matrix)
- Binary responds to LSP initialize on each platform
- Extension finds bundled binary in `server/` directory
- Fallback to download works if bundled binary is missing
- Binary version matches extension version
- Binary is stripped (no debug symbols in release)

---

## 10.4 Auto-Update Mechanism

**Objective:** When a new version of the extension is published, the server binary updates automatically. VS Code handles extension updates; we need to ensure the bundled binary is also updated.

**Steps:**
1. Extension version and server version are always in sync (same release)
2. On activation, check if bundled binary version matches extension version
3. If mismatch (e.g., user manually installed old binary), show update prompt
4. Platform-specific packages (Option B) handle this automatically via marketplace updates

**Code (extension.ts):**
```typescript
async function checkServerVersion(serverPath: string, expectedVersion: string): Promise<boolean> {
    try {
        const { stdout } = await exec(`${serverPath} --version`);
        const serverVersion = stdout.trim().split(' ').pop(); // "borrowscope-lsp 0.2.0" → "0.2.0"
        return serverVersion === expectedVersion;
    } catch {
        return false;
    }
}

async function ensureCorrectVersion(context: vscode.ExtensionContext) {
    const serverPath = getServerPath(context);
    const expectedVersion = context.extension.packageJSON.version;

    if (!await checkServerVersion(serverPath, expectedVersion)) {
        const choice = await vscode.window.showWarningMessage(
            `BorrowScope server version mismatch. Expected ${expectedVersion}.`,
            'Update Server', 'Ignore'
        );
        if (choice === 'Update Server') {
            await downloadServer(context, expectedVersion);
        }
    }
}
```

**Expectation:** Users always run matching extension + server versions. Mismatches are detected and resolved automatically or with a single click.

**Tests for 10.4:**
- Matching versions: no prompt shown
- Mismatched versions: warning shown with update option
- Update downloads correct version
- `--version` flag returns correct version string
- Extension works even if user clicks "Ignore" (graceful degradation)

---

## 10.5 Minimum Rust Toolchain Detection

**Objective:** Detect whether the user has a Rust toolchain installed and whether it meets the minimum version requirement. The server needs `rustc` and `cargo` available for sysroot discovery.

**Steps:**
1. On activation, check for `rustc` and `cargo` in PATH
2. Check version meets minimum (1.70+)
3. If missing, show actionable error with install link
4. If version too old, show upgrade instructions
5. Check for `rust-src` component (needed for sysroot)

**Code (prerequisites.ts):**
```typescript
interface ToolchainStatus {
    rustcFound: boolean;
    rustcVersion: string | null;
    cargoFound: boolean;
    rustSrcInstalled: boolean;
    meetsMinimum: boolean;
}

async function checkToolchain(): Promise<ToolchainStatus> {
    const rustcVersion = await getCommandOutput('rustc --version');
    const cargoExists = await commandExists('cargo');
    const sysroot = await getCommandOutput('rustc --print sysroot');
    const rustSrcPath = path.join(sysroot?.trim() || '', 'lib', 'rustlib', 'src', 'rust');

    const version = rustcVersion?.match(/rustc (\d+\.\d+\.\d+)/)?.[1];
    const meetsMinimum = version ? compareVersions(version, '1.70.0') >= 0 : false;

    return {
        rustcFound: !!rustcVersion,
        rustcVersion: version || null,
        cargoFound: cargoExists,
        rustSrcInstalled: fs.existsSync(rustSrcPath),
        meetsMinimum,
    };
}

async function showToolchainError(status: ToolchainStatus) {
    if (!status.rustcFound) {
        const action = await vscode.window.showErrorMessage(
            'BorrowScope requires Rust. Install it from rustup.rs.',
            'Open rustup.rs'
        );
        if (action) vscode.env.openExternal(vscode.Uri.parse('https://rustup.rs'));
    } else if (!status.meetsMinimum) {
        vscode.window.showErrorMessage(
            `BorrowScope requires Rust 1.70+. Found ${status.rustcVersion}. Run: rustup update`
        );
    } else if (!status.rustSrcInstalled) {
        vscode.window.showErrorMessage(
            'BorrowScope needs rust-src. Run: rustup component add rust-src'
        );
    }
}
```

**Expectation:** Users without Rust get a clear error with a link to install. Users with old Rust get upgrade instructions. Users missing `rust-src` get the exact command to fix it.

**Tests for 10.5:**
- Rust 1.78 detected: no error, extension proceeds
- Rust 1.60 detected: version too old error shown
- No rustc in PATH: install error with rustup.rs link
- Missing rust-src: specific error with `rustup component add` command
- Detection works on Windows (where PATH handling differs)

---

## 10.6 Documentation

**Objective:** Provide comprehensive documentation for users (how to use), contributors (how to build), and maintainers (architecture decisions).

**Documentation structure:**
```
docs/
├── USER_GUIDE.md          # How to use the extension
├── TROUBLESHOOTING.md     # Common issues and fixes
├── ARCHITECTURE.md        # System design for contributors
├── CONTRIBUTING.md        # How to build, test, submit PRs
├── CHANGELOG.md           # Version history
└── FAQ.md                 # Frequently asked questions
```

**USER_GUIDE.md sections:**
1. Installation
2. First-time setup (what to expect during workspace loading)
3. Understanding the ownership graph
4. Reading borrow scope highlights
5. Using the timeline view
6. Keyboard shortcuts reference
7. Configuration options
8. Working alongside rust-analyzer

**TROUBLESHOOTING.md common issues:**
- "Server failed to start" → check Rust toolchain, check binary permissions
- "Types show as {unknown}" → rust-src not installed
- "High memory usage" → reduce `memoryLimit` setting, close unused files
- "Graph is empty" → cursor not inside a function, workspace still loading
- "Decorations not showing" → check `decorations.enabled` setting

**Expectation:** A user can go from install to productive use by reading the User Guide. A contributor can build and test the extension by reading CONTRIBUTING.md.

**Tests for 10.6:**
- All documentation links in README are valid
- Code examples in docs compile/run
- Screenshots in docs match current UI
- CHANGELOG is updated for each release
- No broken internal links between docs

---

## 10.7 Release Pipeline

**Objective:** Automate the entire release process: build server binaries for all platforms, bundle the extension, run tests, publish to marketplace, and create a GitHub release.

**GitHub Actions workflow:**
```yaml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  build-server:
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest, target: x86_64-unknown-linux-gnu }
          - { os: ubuntu-latest, target: aarch64-unknown-linux-gnu }
          - { os: macos-latest, target: x86_64-apple-darwin }
          - { os: macos-latest, target: aarch64-apple-darwin }
          - { os: windows-latest, target: x86_64-pc-windows-msvc }
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: '${{ matrix.target }}' }
      - run: cargo build --release -p borrowscope-lsp --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with:
          name: server-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/borrowscope-lsp*

  build-extension:
    needs: build-server
    runs-on: ubuntu-latest
    strategy:
      matrix:
        platform: [linux-x64, linux-arm64, darwin-x64, darwin-arm64, win32-x64]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with: { node-version: '18' }
      - run: npm ci
        working-directory: borrowscope-vscode
      - run: npm run build -- --production
        working-directory: borrowscope-vscode
      - uses: actions/download-artifact@v4
        with:
          name: server-${{ matrix.platform-to-target }}
          path: borrowscope-vscode/server/
      - run: npx vsce package --target ${{ matrix.platform }}
        working-directory: borrowscope-vscode
      - uses: actions/upload-artifact@v4
        with:
          name: vsix-${{ matrix.platform }}
          path: borrowscope-vscode/*.vsix

  publish:
    needs: build-extension
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - run: npx vsce publish --packagePath vsix-*/*.vsix
        env:
          VSCE_PAT: ${{ secrets.VSCE_PAT }}

  github-release:
    needs: [build-server, build-extension]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
      - uses: softprops/action-gh-release@v1
        with:
          files: |
            server-*/*
            vsix-*/*.vsix
```

**Release checklist (automated):**
1. Tag pushed (`v0.1.0`)
2. Server binaries built for 5 platforms
3. Extension bundled with correct binary per platform
4. Tests pass on all platforms
5. `.vsix` packages created
6. Published to VS Code Marketplace
7. GitHub Release created with binaries and changelogs

**Expectation:** Pushing a version tag triggers the entire release pipeline. Within 30 minutes, the new version is available on the marketplace for all platforms.

**Tests for 10.7:**
- CI builds server for all 5 targets without errors
- Each `.vsix` contains the correct platform binary
- `vsce package` produces valid package (passes validation)
- Published extension installs and activates on each platform
- GitHub Release contains all artifacts
- Version numbers are consistent (extension, server, tag)

---

## 10.T Integration Test Suite

```typescript
suite('Publishing and Distribution Tests', () => {
    test('Extension installs from .vsix', async () => {
        // Install from local .vsix, verify activation
    });

    test('Server binary executes on current platform', async () => {
        const serverPath = getServerPath();
        const { stdout } = await exec(`${serverPath} --version`);
        assert.match(stdout, /borrowscope-lsp \d+\.\d+\.\d+/);
    });

    test('Toolchain detection works', async () => {
        const status = await checkToolchain();
        assert.ok(status.rustcFound);
        assert.ok(status.meetsMinimum);
    });

    test('Extension package size within limits', () => {
        const vsixSize = fs.statSync('borrowscope.vsix').size;
        assert.ok(vsixSize < 40 * 1024 * 1024); // < 40MB
    });

    test('All marketplace assets present', () => {
        assert.ok(fs.existsSync('media/icon.png'));
        assert.ok(fs.existsSync('media/demo.gif'));
        assert.ok(fs.existsSync('README.md'));
        assert.ok(fs.existsSync('CHANGELOG.md'));
    });
});
```
