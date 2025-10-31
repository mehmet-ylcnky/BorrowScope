# Section 8: Development Environment Optimization

## Learning Objectives

By the end of this section, you will:
- Optimize your IDE for Rust development
- Configure debugging tools
- Set up productivity shortcuts
- Understand performance profiling tools
- Create an efficient development workflow
- Master time-saving techniques

## Prerequisites

- Completed Sections 1-7
- IDE installed (VS Code, IntelliJ, or similar)
- Basic familiarity with your chosen IDE

---

## Choosing Your IDE

### VS Code (Recommended for Beginners)

**Pros:**
- ✅ Free and open source
- ✅ Excellent Rust support (rust-analyzer)
- ✅ Large extension ecosystem
- ✅ Cross-platform
- ✅ Lightweight

**Cons:**
- ❌ Less integrated than full IDEs
- ❌ Requires extension configuration

### IntelliJ IDEA / RustRover

**Pros:**
- ✅ Professional IDE features
- ✅ Excellent refactoring tools
- ✅ Integrated debugger
- ✅ Database tools

**Cons:**
- ❌ Heavier resource usage
- ❌ RustRover requires subscription (IntelliJ Community is free)

### Vim/Neovim

**Pros:**
- ✅ Extremely fast
- ✅ Highly customizable
- ✅ Terminal-based
- ✅ Low resource usage

**Cons:**
- ❌ Steep learning curve
- ❌ Requires significant configuration

**For this course:** We'll focus on VS Code, but concepts apply to all IDEs.

---

## Step 1: VS Code Setup

### Install VS Code

Download from: https://code.visualstudio.com/

### Essential Extensions

Install these extensions:

```bash
# rust-analyzer (Rust language server)
code --install-extension rust-lang.rust-analyzer

# CodeLLDB (Debugger)
code --install-extension vadimcn.vscode-lldb

# Even Better TOML
code --install-extension tamasfe.even-better-toml

# Error Lens (Inline errors)
code --install-extension usernamehw.errorlens

# GitLens (Git integration)
code --install-extension eamodio.gitlens

# Crates (Cargo.toml helper)
code --install-extension serayuzgur.crates
```

### Recommended Extensions

```bash
# Better Comments
code --install-extension aaron-bond.better-comments

# Bracket Pair Colorizer
code --install-extension CoenraadS.bracket-pair-colorizer-2

# TODO Highlight
code --install-extension wayou.vscode-todo-highlight

# Markdown All in One
code --install-extension yzhang.markdown-all-in-one
```

---

## Step 2: Configure VS Code

### Create .vscode/settings.json

```json
{
  // ===== Rust Analyzer =====
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": [
    "--all-targets",
    "--all-features"
  ],
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.procMacro.enable": true,
  "rust-analyzer.rustfmt.extraArgs": [
    "+nightly"
  ],
  "rust-analyzer.inlayHints.enable": true,
  "rust-analyzer.inlayHints.chainingHints": true,
  "rust-analyzer.inlayHints.parameterHints": true,
  "rust-analyzer.lens.enable": true,
  "rust-analyzer.lens.run": true,
  "rust-analyzer.lens.debug": true,
  
  // ===== Editor =====
  "editor.formatOnSave": true,
  "editor.formatOnPaste": false,
  "editor.formatOnType": false,
  "editor.rulers": [100],
  "editor.tabSize": 4,
  "editor.insertSpaces": true,
  "editor.detectIndentation": false,
  "editor.renderWhitespace": "boundary",
  "editor.bracketPairColorization.enabled": true,
  "editor.guides.bracketPairs": true,
  "editor.minimap.enabled": true,
  "editor.minimap.renderCharacters": false,
  "editor.suggestSelection": "first",
  "editor.inlineSuggest.enabled": true,
  
  // ===== Rust Specific =====
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer",
    "editor.formatOnSave": true,
    "editor.semanticHighlighting.enabled": true
  },
  
  // ===== TOML =====
  "[toml]": {
    "editor.defaultFormatter": "tamasfe.even-better-toml",
    "editor.formatOnSave": true
  },
  
  // ===== Files =====
  "files.exclude": {
    "**/.git": true,
    "**/target": true,
    "**/.DS_Store": true
  },
  "files.watcherExclude": {
    "**/target/**": true
  },
  "files.trimTrailingWhitespace": true,
  "files.insertFinalNewline": true,
  "files.trimFinalNewlines": true,
  
  // ===== Terminal =====
  "terminal.integrated.defaultProfile.linux": "bash",
  "terminal.integrated.fontSize": 14,
  
  // ===== Git =====
  "git.autofetch": true,
  "git.confirmSync": false,
  "git.enableSmartCommit": true,
  
  // ===== Error Lens =====
  "errorLens.enabledDiagnosticLevels": [
    "error",
    "warning",
    "info"
  ],
  "errorLens.fontSize": "12px",
  
  // ===== Search =====
  "search.exclude": {
    "**/target": true,
    "**/node_modules": true,
    "**/.git": true
  }
}
```

### Understanding Key Settings

#### Rust Analyzer

```json
"rust-analyzer.check.command": "clippy"
```
Runs clippy instead of cargo check for better linting.

```json
"rust-analyzer.inlayHints.enable": true
```
Shows type hints inline:
```rust
let x = 5;  // : i32
```

```json
"rust-analyzer.lens.run": true
```
Adds "Run" and "Debug" buttons above functions.

#### Editor

```json
"editor.formatOnSave": true
```
Automatically formats code when you save.

```json
"editor.rulers": [100]
```
Shows a vertical line at 100 characters (our max line width).

---

## Step 3: Configure Debugging

### Create .vscode/launch.json

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug borrowscope-cli",
      "cargo": {
        "args": [
          "build",
          "--bin=cargo-borrowscope",
          "--package=borrowscope-cli"
        ],
        "filter": {
          "name": "cargo-borrowscope",
          "kind": "bin"
        }
      },
      "args": ["borrowscope", "visualize", "test.rs"],
      "cwd": "${workspaceFolder}"
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug unit tests",
      "cargo": {
        "args": [
          "test",
          "--no-run",
          "--lib",
          "--package=borrowscope-runtime"
        ],
        "filter": {
          "name": "borrowscope-runtime",
          "kind": "lib"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug current test",
      "cargo": {
        "args": [
          "test",
          "--no-run",
          "${fileBasenameNoExtension}"
        ]
      },
      "args": [],
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### Using the Debugger

1. Set breakpoints by clicking left of line numbers
2. Press F5 or click "Run and Debug"
3. Use debug controls:
   - F5: Continue
   - F10: Step Over
   - F11: Step Into
   - Shift+F11: Step Out
   - Shift+F5: Stop

---

## Step 4: Configure Tasks

### Create .vscode/tasks.json

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "cargo build",
      "type": "shell",
      "command": "cargo",
      "args": ["build", "--workspace"],
      "group": {
        "kind": "build",
        "isDefault": true
      },
      "presentation": {
        "reveal": "always",
        "panel": "new"
      },
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "cargo test",
      "type": "shell",
      "command": "cargo",
      "args": ["test", "--workspace"],
      "group": {
        "kind": "test",
        "isDefault": true
      },
      "presentation": {
        "reveal": "always",
        "panel": "new"
      },
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "cargo clippy",
      "type": "shell",
      "command": "cargo",
      "args": ["clippy", "--workspace", "--all-targets", "--all-features"],
      "presentation": {
        "reveal": "always",
        "panel": "new"
      },
      "problemMatcher": ["$rustc"]
    },
    {
      "label": "cargo fmt",
      "type": "shell",
      "command": "cargo",
      "args": ["fmt", "--all"],
      "presentation": {
        "reveal": "silent",
        "panel": "shared"
      }
    },
    {
      "label": "cargo check",
      "type": "shell",
      "command": "cargo",
      "args": ["check", "--workspace"],
      "presentation": {
        "reveal": "always",
        "panel": "shared"
      },
      "problemMatcher": ["$rustc"]
    }
  ]
}
```

### Running Tasks

- Press `Ctrl+Shift+B` (Cmd+Shift+B on Mac) for build
- Press `Ctrl+Shift+P` → "Tasks: Run Task" → Select task

---

## Step 5: Keyboard Shortcuts

### Create .vscode/keybindings.json

```json
[
  {
    "key": "ctrl+shift+b",
    "command": "workbench.action.tasks.build"
  },
  {
    "key": "ctrl+shift+t",
    "command": "workbench.action.tasks.test"
  },
  {
    "key": "ctrl+shift+f",
    "command": "rust-analyzer.run",
    "when": "editorTextFocus && editorLangId == 'rust'"
  },
  {
    "key": "ctrl+shift+d",
    "command": "rust-analyzer.debug",
    "when": "editorTextFocus && editorLangId == 'rust'"
  },
  {
    "key": "ctrl+shift+e",
    "command": "rust-analyzer.expandMacro",
    "when": "editorTextFocus && editorLangId == 'rust'"
  }
]
```

### Essential Shortcuts

**General:**
- `Ctrl+P` - Quick file open
- `Ctrl+Shift+P` - Command palette
- `Ctrl+B` - Toggle sidebar
- `Ctrl+`` - Toggle terminal

**Editing:**
- `Ctrl+D` - Select next occurrence
- `Ctrl+Shift+L` - Select all occurrences
- `Alt+Up/Down` - Move line up/down
- `Ctrl+/` - Toggle comment

**Navigation:**
- `F12` - Go to definition
- `Alt+F12` - Peek definition
- `Shift+F12` - Find all references
- `Ctrl+T` - Go to symbol

**Rust-specific:**
- `Ctrl+Shift+F` - Run current function
- `Ctrl+Shift+D` - Debug current function
- `Ctrl+Shift+E` - Expand macro

---

## Step 6: Code Snippets

### Create .vscode/rust.code-snippets

```json
{
  "Test Function": {
    "prefix": "test",
    "body": [
      "#[test]",
      "fn ${1:test_name}() {",
      "    ${2:// Test code}",
      "}"
    ],
    "description": "Create a test function"
  },
  "Doc Comment": {
    "prefix": "doc",
    "body": [
      "/// ${1:Description}",
      "///",
      "/// # Examples",
      "///",
      "/// ```",
      "/// ${2:// Example code}",
      "/// ```"
    ],
    "description": "Create a documentation comment"
  },
  "Result Function": {
    "prefix": "fnr",
    "body": [
      "pub fn ${1:function_name}(${2:args}) -> Result<${3:T}, ${4:Error}> {",
      "    ${5:// Implementation}",
      "    Ok(${6:value})",
      "}"
    ],
    "description": "Create a function returning Result"
  },
  "Derive Debug": {
    "prefix": "derive",
    "body": [
      "#[derive(Debug, Clone, PartialEq)]"
    ],
    "description": "Common derive attributes"
  }
}
```

### Using Snippets

Type the prefix and press Tab:
- `test` → Creates test function
- `doc` → Creates doc comment
- `fnr` → Creates Result-returning function

---

## Step 7: Performance Profiling Tools

### Install cargo-flamegraph

```bash
cargo install flamegraph
```

### Usage

```bash
# Profile your application
cargo flamegraph --bin cargo-borrowscope

# Opens flamegraph.svg in browser
```

### Install cargo-bloat

```bash
cargo install cargo-bloat
```

### Usage

```bash
# See what takes up space in binary
cargo bloat --release -n 10

# See what takes up space in crate
cargo bloat --release --crates
```

### Install cargo-expand

```bash
cargo install cargo-expand
```

### Usage

```bash
# See macro expansion
cargo expand --lib

# Expand specific module
cargo expand runtime::tracker
```

---

## Step 8: Productivity Tools

### Install cargo-watch

```bash
cargo install cargo-watch
```

### Usage

```bash
# Auto-run tests on file change
cargo watch -x test

# Auto-run clippy
cargo watch -x clippy

# Chain commands
cargo watch -x check -x test -x run
```

### Install cargo-edit

```bash
cargo install cargo-edit
```

### Usage

```bash
# Add dependency
cargo add serde

# Add dev dependency
cargo add --dev criterion

# Remove dependency
cargo rm serde

# Upgrade dependencies
cargo upgrade
```

### Install cargo-tree

```bash
# Usually included with Cargo
cargo tree
```

### Usage

```bash
# Show dependency tree
cargo tree

# Show specific package
cargo tree -p serde

# Show duplicates
cargo tree --duplicates
```

---

## Step 9: Shell Aliases

Add to your `.bashrc` or `.zshrc`:

```bash
# Cargo shortcuts
alias cb='cargo build'
alias ct='cargo test'
alias cr='cargo run'
alias cc='cargo check'
alias ccl='cargo clippy'
alias cf='cargo fmt'

# Cargo workspace
alias cbw='cargo build --workspace'
alias ctw='cargo test --workspace'
alias ccw='cargo check --workspace'

# Cargo with options
alias cbr='cargo build --release'
alias ctr='cargo test --release'
alias crr='cargo run --release'

# BorrowScope specific
alias bs='cargo run -p borrowscope-cli -- borrowscope'
alias bsv='cargo run -p borrowscope-cli -- borrowscope visualize'
alias bsr='cargo run -p borrowscope-cli -- borrowscope run'

# Git shortcuts
alias gs='git status'
alias ga='git add'
alias gc='git commit'
alias gp='git push'
alias gl='git log --oneline --graph --decorate'

# Development workflow
alias dev='cargo watch -x check -x test'
alias check-all='cargo fmt --all && cargo clippy --workspace && cargo test --workspace'
```

Reload shell:
```bash
source ~/.bashrc  # or ~/.zshrc
```

---

## Step 10: Optimized Workflow

### Morning Routine

```bash
# 1. Update Rust
rustup update

# 2. Pull latest changes
git pull

# 3. Check everything compiles
cargo check --workspace

# 4. Run tests
cargo test --workspace

# 5. Start development
cargo watch -x check -x test
```

### Before Committing

```bash
# Run all checks
make check  # or check-all alias

# Or manually:
cargo fmt --all
cargo clippy --workspace
cargo test --workspace
cargo doc --workspace --no-deps
```

### Development Loop

```bash
# Terminal 1: Auto-check
cargo watch -x check

# Terminal 2: Development
# Edit code in VS Code

# Terminal 3: Manual tests
cargo test specific_test
```

---

## Best Practices Summary

### IDE Configuration

✅ **Install rust-analyzer** - Essential for Rust development  
✅ **Enable format on save** - Consistent code style  
✅ **Configure debugger** - Faster bug fixing  
✅ **Use code snippets** - Speed up common tasks  
✅ **Set up tasks** - One-click build/test  

### Productivity

✅ **Learn keyboard shortcuts** - Faster navigation  
✅ **Use cargo-watch** - Auto-run on changes  
✅ **Create shell aliases** - Shorter commands  
✅ **Profile regularly** - Find bottlenecks  
✅ **Expand macros** - Understand generated code  

### Workflow

✅ **Check before commit** - Catch issues early  
✅ **Use multiple terminals** - Parallel workflows  
✅ **Leverage IDE features** - Refactoring, navigation  
✅ **Document as you go** - Don't defer docs  
✅ **Test continuously** - cargo watch -x test  

---

## Exercises

### Exercise 1: Configure Your IDE

1. Install VS Code and extensions
2. Create .vscode/settings.json
3. Test rust-analyzer features
4. Set up debugging

### Exercise 2: Test Debugging

1. Add a breakpoint in borrowscope-runtime
2. Run debugger
3. Step through code
4. Inspect variables

### Exercise 3: Create Custom Snippets

Create a snippet for your most common code pattern.

### Exercise 4: Profile the Code

```bash
cargo install flamegraph
cargo flamegraph --bin cargo-borrowscope
```

Examine the flamegraph.

---

## Key Takeaways

### Development Environment

✅ **IDE** - VS Code with rust-analyzer recommended  
✅ **Extensions** - CodeLLDB, Error Lens, GitLens  
✅ **Configuration** - Format on save, inlay hints  
✅ **Debugging** - Breakpoints, step through code  
✅ **Tasks** - One-click build/test/clippy  

### Productivity Tools

✅ **cargo-watch** - Auto-run on changes  
✅ **cargo-expand** - See macro expansions  
✅ **cargo-flamegraph** - Performance profiling  
✅ **cargo-bloat** - Binary size analysis  
✅ **cargo-edit** - Manage dependencies  

### Workflow Optimization

✅ **Keyboard shortcuts** - Faster navigation  
✅ **Shell aliases** - Shorter commands  
✅ **Multiple terminals** - Parallel work  
✅ **Continuous testing** - Catch bugs early  
✅ **Regular profiling** - Maintain performance  

---

## Congratulations! 🎉

You've completed **Chapter 1: Foundation & Project Setup**!

You now have:
- ✅ A complete Cargo workspace
- ✅ Three crates (macro, runtime, CLI)
- ✅ Git version control configured
- ✅ CI/CD pipeline with GitHub Actions
- ✅ Rust toolchain configured
- ✅ Comprehensive documentation
- ✅ Optimized development environment

**You're ready to start building BorrowScope!**

---

## What's Next?

In **Chapter 2: Procedural Macros Fundamentals**, we'll:
- Deep dive into procedural macros
- Learn syn and quote in detail
- Parse Rust syntax trees
- Generate code programmatically
- Build the `#[trace_borrow]` macro

Get ready to write code that writes code! 🚀

---

**Previous Section:** [07-project-documentation-structure.md](./07-project-documentation-structure.md)  
**Next Chapter:** [Chapter 2: Procedural Macros Fundamentals](../chapter-02/09-introduction-to-procedural-macros.md)

**Chapter Progress:** 8/8 sections complete ⬛⬛⬛⬛⬛⬛⬛⬛ ✅

---

*"A well-configured environment is half the battle won." 💪*
