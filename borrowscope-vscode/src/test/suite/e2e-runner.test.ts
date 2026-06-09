import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";

// Mock vscode before importing the module
const vscode = require("vscode");

import { resolveAnalyzerPath, spawnProcess, runE2EPipeline, E2ERunnerOptions } from "../../e2e-runner";

describe("E2E Runner", () => {
  describe("resolveAnalyzerPath", () => {
    it("should return null when no LSP path configured and no target directory", () => {
      vscode.workspace.getConfiguration = () => ({
        get: (key: string, def: any) => def,
      });
      vscode.workspace.workspaceFolders = undefined;
      const result = resolveAnalyzerPath("");
      assert.strictEqual(result, null);
    });

    it("should find analyzer next to configured LSP path", () => {
      const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-test-"));
      const analyzerPath = path.join(tmpDir, "borrowscope-analyzer");
      fs.writeFileSync(analyzerPath, "#!/bin/sh\necho test", { mode: 0o755 });

      vscode.workspace.getConfiguration = () => ({
        get: (key: string, def: any) => {
          if (key === "path") return path.join(tmpDir, "borrowscope-lsp");
          return def;
        },
      });

      const result = resolveAnalyzerPath("");
      assert.strictEqual(result, analyzerPath);

      fs.unlinkSync(analyzerPath);
      fs.rmdirSync(tmpDir);
    });

    it("should find analyzer in target/release when workspace exists", () => {
      const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-test-"));
      const releaseDir = path.join(tmpDir, "target", "release");
      fs.mkdirSync(releaseDir, { recursive: true });
      const analyzerPath = path.join(releaseDir, "borrowscope-analyzer");
      fs.writeFileSync(analyzerPath, "#!/bin/sh\necho test", { mode: 0o755 });

      vscode.workspace.getConfiguration = () => ({
        get: (key: string, def: any) => def,
      });
      vscode.workspace.workspaceFolders = [{ uri: { fsPath: tmpDir } }];

      const result = resolveAnalyzerPath("");
      assert.strictEqual(result, analyzerPath);

      fs.unlinkSync(analyzerPath);
      fs.rmdirSync(releaseDir);
      fs.rmdirSync(path.join(tmpDir, "target"));
      fs.rmdirSync(tmpDir);
    });

    it("should prefer debug build when release not available", () => {
      const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-test-"));
      const debugDir = path.join(tmpDir, "target", "debug");
      fs.mkdirSync(debugDir, { recursive: true });
      const analyzerPath = path.join(debugDir, "borrowscope-analyzer");
      fs.writeFileSync(analyzerPath, "#!/bin/sh\necho test", { mode: 0o755 });

      vscode.workspace.getConfiguration = () => ({
        get: (key: string, def: any) => def,
      });
      vscode.workspace.workspaceFolders = [{ uri: { fsPath: tmpDir } }];

      const result = resolveAnalyzerPath("");
      assert.strictEqual(result, analyzerPath);

      fs.unlinkSync(analyzerPath);
      fs.rmdirSync(debugDir);
      fs.rmdirSync(path.join(tmpDir, "target"));
      fs.rmdirSync(tmpDir);
    });
  });

  describe("spawnProcess", () => {
    it("should capture stdout from a successful command", async () => {
      const result = await spawnProcess("echo", ["hello world"], os.tmpdir());
      assert.strictEqual(result.code, 0);
      assert.ok(result.stdout.includes("hello"));
    });

    it("should return non-zero exit code on failure", async () => {
      const result = await spawnProcess("false", [], os.tmpdir());
      assert.notStrictEqual(result.code, 0);
    });

    it("should capture stderr from failing command", async () => {
      const result = await spawnProcess("ls", ["/nonexistent_path_xyz"], os.tmpdir());
      assert.notStrictEqual(result.code, 0);
      assert.ok(result.stderr.length > 0);
    });

    it("should call onOutput callback for each line", async () => {
      const lines: string[] = [];
      await spawnProcess("echo", ["-e", "line1\\nline2\\nline3"], os.tmpdir(), (line: string) => lines.push(line));
      assert.ok(lines.length >= 1);
    });

    it("should handle command not found gracefully", async () => {
      const result = await spawnProcess("nonexistent_command_xyz_123", [], os.tmpdir());
      assert.notStrictEqual(result.code, 0);
    });
  });

  describe("runE2EPipeline", () => {
    let tmpDir: string;
    let progressMessages: string[];
    let mockProgress: any;
    let mockToken: any;

    beforeEach(() => {
      tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-e2e-"));
      progressMessages = [];
      mockProgress = {
        report: (value: { message?: string; increment?: number }) => {
          if (value.message) progressMessages.push(value.message);
        },
      };
      mockToken = { isCancellationRequested: false };
    });

    afterEach(() => {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    });

    it("should fail when analyzer binary does not exist", async () => {
      const options: E2ERunnerOptions = {
        workspaceRoot: tmpDir,
        analyzerPath: "/nonexistent/borrowscope-analyzer",
      };
      const result = await runE2EPipeline(options, mockProgress, mockToken);
      assert.strictEqual(result.success, false);
      assert.ok(result.error!.includes("failed"));
    });

    it("should fail when analyzer produces non-zero exit code", async () => {
      // Create a fake analyzer that fails
      const fakeAnalyzer = path.join(tmpDir, "fake-analyzer");
      fs.writeFileSync(fakeAnalyzer, "#!/bin/sh\nexit 1", { mode: 0o755 });

      const options: E2ERunnerOptions = {
        workspaceRoot: tmpDir,
        analyzerPath: fakeAnalyzer,
      };
      const result = await runE2EPipeline(options, mockProgress, mockToken);
      assert.strictEqual(result.success, false);
      assert.ok(result.error!.includes("Analyzer failed"));
      assert.ok(result.analyzerDuration > 0);
    });

    it("should fail when analyzer succeeds but type-info.json not produced", async () => {
      // Create a fake analyzer that exits 0 but produces nothing
      const fakeAnalyzer = path.join(tmpDir, "fake-analyzer");
      fs.writeFileSync(fakeAnalyzer, "#!/bin/sh\necho 'done'", { mode: 0o755 });

      const options: E2ERunnerOptions = {
        workspaceRoot: tmpDir,
        analyzerPath: fakeAnalyzer,
      };
      const result = await runE2EPipeline(options, mockProgress, mockToken);
      assert.strictEqual(result.success, false);
      assert.ok(result.error!.includes("type-info.json was not produced"));
    });

    it("should fail when cargo run fails after successful analysis", async () => {
      // Create fake analyzer that produces type-info.json
      const bsDir = path.join(tmpDir, ".borrowscope");
      fs.mkdirSync(bsDir, { recursive: true });
      const fakeAnalyzer = path.join(tmpDir, "fake-analyzer");
      fs.writeFileSync(fakeAnalyzer,
        `#!/bin/sh\nmkdir -p "${bsDir}"\necho '{}' > "${path.join(bsDir, "type-info.json")}"`,
        { mode: 0o755 });

      const options: E2ERunnerOptions = {
        workspaceRoot: "/nonexistent_workspace_for_cargo",
        analyzerPath: fakeAnalyzer,
      };
      // Override workspaceRoot for analyzer but cargo will fail because no Cargo.toml
      const opts2: E2ERunnerOptions = {
        workspaceRoot: tmpDir,
        analyzerPath: fakeAnalyzer,
      };
      const result = await runE2EPipeline(opts2, mockProgress, mockToken);
      assert.strictEqual(result.success, false);
      assert.ok(result.error!.includes("cargo run failed"));
    });

    it("should succeed and count events when full pipeline works", async () => {
      // Create fake analyzer that produces type-info.json
      const bsDir = path.join(tmpDir, ".borrowscope");
      fs.mkdirSync(bsDir, { recursive: true });
      const fakeAnalyzer = path.join(tmpDir, "fake-analyzer");
      fs.writeFileSync(fakeAnalyzer,
        `#!/bin/sh\nmkdir -p "${bsDir}"\necho '{}' > "${path.join(bsDir, "type-info.json")}"`,
        { mode: 0o755 });

      // Create a fake Cargo project that produces events
      fs.writeFileSync(path.join(tmpDir, "Cargo.toml"), `[package]\nname = "test"\nversion = "0.1.0"\nedition = "2021"\n`);
      fs.mkdirSync(path.join(tmpDir, "src"));
      // main.rs that writes a fake events.json
      fs.writeFileSync(path.join(tmpDir, "src", "main.rs"),
        `fn main() {
  std::fs::create_dir_all(".borrowscope").ok();
  std::fs::write(".borrowscope/events.json", "[{\\"New\\":{\\"timestamp\\":1,\\"var_name\\":\\"x\\",\\"var_id\\":\\"x_0\\",\\"type_name\\":\\"i32\\"}}]").unwrap();
}`);

      const options: E2ERunnerOptions = {
        workspaceRoot: tmpDir,
        analyzerPath: fakeAnalyzer,
      };
      const result = await runE2EPipeline(options, mockProgress, mockToken);
      assert.strictEqual(result.success, true);
      assert.strictEqual(result.eventsCount, 1);
      assert.ok(result.analyzerDuration >= 0);
      assert.ok(result.runDuration >= 0);
      assert.ok(result.eventsPath.includes("events.json"));
    });

    it("should report progress messages in order", async () => {
      const bsDir = path.join(tmpDir, ".borrowscope");
      fs.mkdirSync(bsDir, { recursive: true });
      const fakeAnalyzer = path.join(tmpDir, "fake-analyzer");
      fs.writeFileSync(fakeAnalyzer,
        `#!/bin/sh\nmkdir -p "${bsDir}"\necho '{}' > "${path.join(bsDir, "type-info.json")}"`,
        { mode: 0o755 });
      // No Cargo.toml so cargo run will fail, but we still get the first progress message
      const options: E2ERunnerOptions = { workspaceRoot: tmpDir, analyzerPath: fakeAnalyzer };
      await runE2EPipeline(options, mockProgress, mockToken);

      assert.ok(progressMessages.includes("Analyzing types..."));
      assert.ok(progressMessages.includes("Compiling & running..."));
    });

    it("should respect cancellation token before cargo run", async () => {
      const bsDir = path.join(tmpDir, ".borrowscope");
      fs.mkdirSync(bsDir, { recursive: true });
      const fakeAnalyzer = path.join(tmpDir, "fake-analyzer");
      fs.writeFileSync(fakeAnalyzer,
        `#!/bin/sh\nmkdir -p "${bsDir}"\necho '{}' > "${path.join(bsDir, "type-info.json")}"`,
        { mode: 0o755 });

      // Cancel after analyzer
      const cancelToken = { isCancellationRequested: false };
      const cancelProgress = {
        report: () => { cancelToken.isCancellationRequested = true; },
      };

      const options: E2ERunnerOptions = { workspaceRoot: tmpDir, analyzerPath: fakeAnalyzer };
      const result = await runE2EPipeline(options, cancelProgress as any, cancelToken as any);
      assert.strictEqual(result.success, false);
      assert.strictEqual(result.error, "Cancelled");
    });

    it("should handle malformed events.json gracefully", async () => {
      const bsDir = path.join(tmpDir, ".borrowscope");
      fs.mkdirSync(bsDir, { recursive: true });
      const fakeAnalyzer = path.join(tmpDir, "fake-analyzer");
      fs.writeFileSync(fakeAnalyzer,
        `#!/bin/sh\nmkdir -p "${bsDir}"\necho '{}' > "${path.join(bsDir, "type-info.json")}"`,
        { mode: 0o755 });

      // Cargo project that produces malformed JSON
      fs.writeFileSync(path.join(tmpDir, "Cargo.toml"), `[package]\nname = "test"\nversion = "0.1.0"\nedition = "2021"\n`);
      fs.mkdirSync(path.join(tmpDir, "src"), { recursive: true });
      fs.writeFileSync(path.join(tmpDir, "src", "main.rs"),
        `fn main() { std::fs::create_dir_all(".borrowscope").ok(); std::fs::write(".borrowscope/events.json", "not valid json").unwrap(); }`);

      const options: E2ERunnerOptions = { workspaceRoot: tmpDir, analyzerPath: fakeAnalyzer };
      const result = await runE2EPipeline(options, mockProgress, mockToken);
      // Should still succeed (cargo run succeeded) but events count is 0
      assert.strictEqual(result.success, true);
      assert.strictEqual(result.eventsCount, 0);
    });

    it("should log output to the provided output channel", async () => {
      const logged: string[] = [];
      const mockChannel = { appendLine: (msg: string) => logged.push(msg) } as any;

      const fakeAnalyzer = path.join(tmpDir, "fake-analyzer");
      fs.writeFileSync(fakeAnalyzer, "#!/bin/sh\nexit 1", { mode: 0o755 });

      const options: E2ERunnerOptions = {
        workspaceRoot: tmpDir,
        analyzerPath: fakeAnalyzer,
        outputChannel: mockChannel,
      };
      await runE2EPipeline(options, mockProgress, mockToken);
      assert.ok(logged.some(l => l.includes("[E2E]")));
      assert.ok(logged.some(l => l.includes("Running analyzer")));
    });
  });
});
