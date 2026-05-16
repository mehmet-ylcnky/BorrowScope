import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import { RuntimeEvent } from "./runtime-types";

const DEFAULT_EVENT_FILE = ".borrowscope/events.json";

export class RuntimeWatcher implements vscode.Disposable {
  private watcher: vscode.FileSystemWatcher | undefined;
  private events: RuntimeEvent[] = [];
  private _onEventsUpdated = new vscode.EventEmitter<RuntimeEvent[]>();
  public readonly onEventsUpdated = this._onEventsUpdated.event;
  private _stale = false;
  private _lastLoadTime: number = 0;
  private _eventFilePath: string = "";
  private disposables: vscode.Disposable[] = [];

  start(workspaceRoot: string): void {
    const config = vscode.workspace.getConfiguration("borrowscope.runtime");
    const relPath = config.get<string>("filePath", DEFAULT_EVENT_FILE);
    this._eventFilePath = path.join(workspaceRoot, relPath);

    const pattern = new vscode.RelativePattern(workspaceRoot, relPath);
    this.watcher = vscode.workspace.createFileSystemWatcher(pattern);

    this.watcher.onDidChange((uri) => this.loadEvents(uri.fsPath), null, this.disposables);
    this.watcher.onDidCreate((uri) => this.loadEvents(uri.fsPath), null, this.disposables);
    this.watcher.onDidDelete(() => this.clearEvents(), null, this.disposables);

    // Mark stale when source files change
    const rustWatcher = vscode.workspace.createFileSystemWatcher("**/*.rs");
    rustWatcher.onDidChange(() => { this._stale = true; }, null, this.disposables);
    this.disposables.push(rustWatcher);

    // Load existing file
    if (fs.existsSync(this._eventFilePath)) {
      this.loadEvents(this._eventFilePath);
    }
  }

  private loadEvents(filePath: string): void {
    try {
      const content = fs.readFileSync(filePath, "utf8");
      if (!content.trim()) { this.clearEvents(); return; }
      const parsed = JSON.parse(content);
      this.events = Array.isArray(parsed) ? parsed : [];
      this._stale = false;
      this._lastLoadTime = Date.now();
      this._onEventsUpdated.fire(this.events);
    } catch {
      // Invalid JSON — ignore until next write
    }
  }

  private clearEvents(): void {
    this.events = [];
    this._stale = false;
    this._lastLoadTime = 0;
    this._onEventsUpdated.fire(this.events);
  }

  getEvents(): RuntimeEvent[] { return this.events; }
  get eventCount(): number { return this.events.length; }
  get isStale(): boolean { return this._stale; }
  get lastLoadTime(): number { return this._lastLoadTime; }
  get filePath(): string { return this._eventFilePath; }

  dispose(): void {
    this.watcher?.dispose();
    this._onEventsUpdated.dispose();
    this.disposables.forEach((d) => d.dispose());
  }
}
