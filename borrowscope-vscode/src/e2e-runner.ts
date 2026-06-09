import * as vscode from "vscode";
import * as cp from "child_process";
import * as path from "path";
import * as fs from "fs";

export interface E2ERunnerOptions {
  workspaceRoot: string;
  analyzerPath: string;
  outputChannel?: vscode.OutputChannel;
}

export interface E2EResult {
  success: boolean;
  analyzerDuration: number;
  runDuration: number;
  eventsCount: number;
  eventsPath: string;
  error?: string;
}

interface SpawnResult {
  code: number;
  stdout: string;
  stderr: string;
}

/**
 * Resolve the analyzer binary path.
 * Checks: config setting > sibling to LSP binary > cargo workspace target.
 */
export function resolveAnalyzerPath(extensionPath: string): string | null {
  // 1. Check if LSP server path is configured (analyzer is next to it)
  const lspPath = vscode.workspace
    .getConfiguration("borrowscope.server")
    .get<string>("path", "");

  if (lspPath) {
    const dir = path.dirname(lspPath);
    const analyzerPath = path.join(dir, "borrowscope-analyzer");
    if (fs.existsSync(analyzerPath)) return analyzerPath;
  }

  // 2. Check workspace root for target/release
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (workspaceFolders) {
    const root = workspaceFolders[0].uri.fsPath;
    const releasePath = path.join(root, "target", "release", "borrowscope-analyzer");
    if (fs.existsSync(releasePath)) return releasePath;
    const debugPath = path.join(root, "target", "debug", "borrowscope-analyzer");
    if (fs.existsSync(debugPath)) return debugPath;
  }

  return null;
}

/**
 * Spawn a child process and return stdout/stderr/code.
 */
export function spawnProcess(
  command: string,
  args: string[],
  cwd: string,
  onOutput?: (line: string) => void
): Promise<SpawnResult> {
  return new Promise((resolve) => {
    const proc = cp.spawn(command, args, { cwd, shell: true });
    let stdout = "";
    let stderr = "";

    proc.stdout?.on("data", (data: Buffer) => {
      const text = data.toString();
      stdout += text;
      if (onOutput) text.split("\n").filter(Boolean).forEach(onOutput);
    });

    proc.stderr?.on("data", (data: Buffer) => {
      const text = data.toString();
      stderr += text;
      if (onOutput) text.split("\n").filter(Boolean).forEach(onOutput);
    });

    proc.on("close", (code) => {
      resolve({ code: code ?? 1, stdout, stderr });
    });

    proc.on("error", (err) => {
      resolve({ code: 1, stdout, stderr: err.message });
    });
  });
}

/**
 * Run the full E2E pipeline:
 * 1. Run borrowscope-analyzer to generate type-info.json
 * 2. Run cargo run on the workspace to produce events.json
 */
export async function runE2EPipeline(
  options: E2ERunnerOptions,
  progress: vscode.Progress<{ message?: string; increment?: number }>,
  token: vscode.CancellationToken
): Promise<E2EResult> {
  const { workspaceRoot, analyzerPath, outputChannel } = options;
  const log = (msg: string) => outputChannel?.appendLine(`[E2E] ${msg}`);

  const eventsPath = path.join(workspaceRoot, ".borrowscope", "events.json");

  // Step 1: Run analyzer
  progress.report({ message: "Analyzing types...", increment: 0 });
  log(`Running analyzer: ${analyzerPath} ${workspaceRoot}`);

  const analyzerStart = Date.now();
  const analyzerResult = await spawnProcess(analyzerPath, [workspaceRoot], workspaceRoot, log);
  const analyzerDuration = Date.now() - analyzerStart;

  if (token.isCancellationRequested) {
    return { success: false, analyzerDuration, runDuration: 0, eventsCount: 0, eventsPath, error: "Cancelled" };
  }

  if (analyzerResult.code !== 0) {
    const error = `Analyzer failed (exit ${analyzerResult.code}): ${analyzerResult.stderr.slice(0, 200)}`;
    log(error);
    return { success: false, analyzerDuration, runDuration: 0, eventsCount: 0, eventsPath, error };
  }

  // Verify type-info.json was produced
  const typeInfoPath = path.join(workspaceRoot, ".borrowscope", "type-info.json");
  if (!fs.existsSync(typeInfoPath)) {
    const error = "Analyzer completed but type-info.json was not produced";
    log(error);
    return { success: false, analyzerDuration, runDuration: 0, eventsCount: 0, eventsPath, error };
  }

  log(`Analyzer complete in ${analyzerDuration}ms`);
  progress.report({ message: "Compiling & running...", increment: 50 });

  // Step 2: Run the project (cargo run)
  if (token.isCancellationRequested) {
    return { success: false, analyzerDuration, runDuration: 0, eventsCount: 0, eventsPath, error: "Cancelled" };
  }

  const runStart = Date.now();
  const runResult = await spawnProcess("cargo", ["run"], workspaceRoot, log);
  const runDuration = Date.now() - runStart;

  if (runResult.code !== 0) {
    const error = `cargo run failed (exit ${runResult.code}): ${runResult.stderr.slice(0, 200)}`;
    log(error);
    return { success: false, analyzerDuration, runDuration, eventsCount: 0, eventsPath, error };
  }

  // Check if events were produced
  let eventsCount = 0;
  if (fs.existsSync(eventsPath)) {
    try {
      const content = fs.readFileSync(eventsPath, "utf-8");
      const events = JSON.parse(content);
      eventsCount = Array.isArray(events) ? events.length : 0;
    } catch {
      eventsCount = 0;
    }
  }

  log(`Run complete in ${runDuration}ms, ${eventsCount} events produced`);
  progress.report({ message: `Done! ${eventsCount} events loaded`, increment: 100 });

  return { success: true, analyzerDuration, runDuration, eventsCount, eventsPath };
}

/**
 * Execute the full pipeline with VS Code progress UI.
 */
export async function executeE2E(outputChannel?: vscode.OutputChannel, statusBar?: E2EStatusBar): Promise<void> {
  const workspaceFolders = vscode.workspace.workspaceFolders;
  if (!workspaceFolders || workspaceFolders.length === 0) {
    vscode.window.showErrorMessage("BorrowScope: No workspace folder open");
    return;
  }

  const workspaceRoot = workspaceFolders[0].uri.fsPath;
  const extensionPath = "";

  const analyzerPath = resolveAnalyzerPath(extensionPath);
  if (!analyzerPath) {
    vscode.window.showErrorMessage(
      "BorrowScope: Cannot find borrowscope-analyzer binary. " +
      "Ensure it is built (cargo build -p borrowscope-analyzer --release) " +
      "and the LSP server path is configured."
    );
    return;
  }

  statusBar?.setRunning("Analyzing...");

  const result = await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "BorrowScope: Running instrumented pipeline",
      cancellable: true,
    },
    async (progress, token) => {
      const pipelineProgress: vscode.Progress<{ message?: string; increment?: number }> = {
        report: (value) => {
          progress.report(value);
          if (value.message && statusBar) {
            statusBar.setRunning(value.message.replace("...", ""));
          }
        },
      };
      return runE2EPipeline({ workspaceRoot, analyzerPath, outputChannel }, pipelineProgress, token);
    }
  );

  if (result.success) {
    statusBar?.setSuccess(result.eventsCount);

    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    if (!config.get<boolean>("enabled")) {
      await config.update("enabled", true, vscode.ConfigurationTarget.Workspace);
    }

    vscode.window.showInformationMessage(
      `BorrowScope: Pipeline complete! ` +
      `Analyzer: ${(result.analyzerDuration / 1000).toFixed(1)}s, ` +
      `Run: ${(result.runDuration / 1000).toFixed(1)}s, ` +
      `${result.eventsCount} events captured.`
    );
  } else {
    statusBar?.setError(result.error || "Unknown error");
    vscode.window.showErrorMessage(`BorrowScope: ${result.error}`);
  }
}

/**
 * Status bar button for the E2E pipeline.
 * Shows: ▶ BorrowScope (idle) | ⏳ Analyzing... (running) | ✓ N events (done) | ✗ Failed (error)
 */
export class E2EStatusBar implements vscode.Disposable {
  private item: vscode.StatusBarItem;
  private timeout: NodeJS.Timeout | undefined;

  constructor() {
    this.item = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 50);
    this.item.command = "borrowscope.runInstrumented";
    this.setIdle();
    this.item.show();
  }

  setIdle(): void {
    this.item.text = "$(play) BorrowScope";
    this.item.tooltip = "Run Instrumented Pipeline (Analyze + Build + Run)";
    this.item.color = undefined;
  }

  setRunning(stage: string): void {
    this.item.text = `$(sync~spin) ${stage}`;
    this.item.tooltip = "Pipeline running...";
    this.item.color = new vscode.ThemeColor("statusBarItem.warningForeground");
  }

  setSuccess(eventsCount: number): void {
    this.item.text = `$(check) ${eventsCount} events`;
    this.item.tooltip = "Pipeline complete. Click to run again.";
    this.item.color = new vscode.ThemeColor("statusBarItem.prominentForeground");
    this.resetAfterDelay(8000);
  }

  setError(message: string): void {
    this.item.text = "$(error) Pipeline failed";
    this.item.tooltip = message;
    this.item.color = new vscode.ThemeColor("statusBarItem.errorForeground");
    this.resetAfterDelay(10000);
  }

  private resetAfterDelay(ms: number): void {
    if (this.timeout) clearTimeout(this.timeout);
    this.timeout = setTimeout(() => this.setIdle(), ms);
  }

  dispose(): void {
    if (this.timeout) clearTimeout(this.timeout);
    this.item.dispose();
  }
}
