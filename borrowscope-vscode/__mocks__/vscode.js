// Minimal vscode mock for unit testing outside Extension Development Host
const decorationTypes = [];

class Position {
  constructor(line, character) { this.line = line; this.character = character; }
}
class Range {
  constructor(startLineOrPos, startCharOrEnd, endLine, endChar) {
    if (typeof startLineOrPos === 'number') {
      this.start = new Position(startLineOrPos, startCharOrEnd);
      this.end = new Position(endLine, endChar);
    } else {
      this.start = startLineOrPos;
      this.end = startCharOrEnd;
    }
  }
}
class Selection extends Range {}

class Uri {
  constructor(path) { this.fsPath = path; this.path = path; }
  toString() { return `file://${this.path}`; }
  static parse(str) { return new Uri(str.replace('file://', '')); }
  static file(path) { return new Uri(path); }
  static joinPath(base, ...segments) { return new Uri(base.fsPath + '/' + segments.join('/')); }
}

// WebView mock
class WebviewPanel {
  constructor(viewType, title, column, options) {
    this.viewType = viewType;
    this.title = title;
    this.viewColumn = column;
    this.options = options;
    this.webview = {
      html: '',
      _lastMessage: null,
      postMessage: function(msg) { this._lastMessage = msg; return Promise.resolve(true); },
      onDidReceiveMessage: (handler) => { this._messageHandler = handler; return { dispose: () => {} }; },
      asWebviewUri: (uri) => uri,
    };
    this._disposed = false;
    this._lastMessage = null;
    this._messageHandler = null;
    this._onDidDispose = null;
  }
  reveal(column) { this.viewColumn = column; }
  onDidDispose(handler) { this._onDidDispose = handler; return { dispose: () => {} }; }
  dispose() {
    if (this._disposed) return;
    this._disposed = true;
    if (this._onDidDispose) this._onDidDispose();
  }
}

module.exports = {
  Position,
  Range,
  Selection,
  Uri,
  ViewColumn: { One: 1, Two: 2, Beside: -2 },
  TextEditorRevealType: { InCenter: 2 },
  window: {
    createOutputChannel: (name) => ({
      appendLine: () => {},
      clear: () => {},
      show: () => {},
      dispose: () => {},
    }),
    showInformationMessage: () => Promise.resolve(undefined),
    showWarningMessage: () => Promise.resolve(undefined),
    showErrorMessage: () => Promise.resolve(undefined),
    createTextEditorDecorationType: (options) => {
      const dt = { options, disposed: false, dispose: () => { dt.disposed = true; } };
      decorationTypes.push(dt);
      return dt;
    },
    createWebviewPanel: (viewType, title, column, options) => {
      return new WebviewPanel(viewType, title, column, options);
    },
    withProgress: (opts, task) => task({ report: () => {} }),
    activeTextEditor: undefined,
    onDidChangeActiveTextEditor: () => ({ dispose: () => {} }),
  },
  workspace: {
    getConfiguration: (section) => ({
      get: (key, defaultValue) => defaultValue,
      update: () => Promise.resolve(),
    }),
    createFileSystemWatcher: () => ({ dispose: () => {} }),
    openTextDocument: () => Promise.resolve({ uri: new Uri('/tmp/test.rs') }),
    onDidOpenTextDocument: () => ({ dispose: () => {} }),
    onDidChangeTextDocument: () => ({ dispose: () => {} }),
  },
  commands: {
    registerCommand: (id, handler) => ({ dispose: () => {} }),
  },
  env: {
    clipboard: { writeText: () => Promise.resolve() },
  },
  ConfigurationTarget: { Global: 1, Workspace: 2 },
  ProgressLocation: { Notification: 15 },
  OverviewRulerLane: { Left: 1, Center: 2, Right: 4, Full: 7 },
  // Test helpers
  __test__: { decorationTypes, WebviewPanel },
};
