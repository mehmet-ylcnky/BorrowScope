// Minimal vscode mock for unit testing outside Extension Development Host
const decorationTypes = [];
const appliedDecorations = new Map();

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

module.exports = {
  Position,
  Range,
  window: {
    createOutputChannel: (name) => ({
      appendLine: () => {},
      dispose: () => {},
    }),
    showInformationMessage: () => {},
    showErrorMessage: () => {},
    createTextEditorDecorationType: (options) => {
      const dt = { options, disposed: false, dispose: () => { dt.disposed = true; } };
      decorationTypes.push(dt);
      return dt;
    },
    withProgress: (opts, task) => task({ report: () => {} }),
  },
  workspace: {
    getConfiguration: (section) => ({
      get: (key, defaultValue) => defaultValue,
      update: () => Promise.resolve(),
    }),
    createFileSystemWatcher: () => ({ dispose: () => {} }),
  },
  commands: {
    registerCommand: (id, handler) => ({ dispose: () => {} }),
  },
  ConfigurationTarget: { Global: 1, Workspace: 2 },
  ProgressLocation: { Notification: 15 },
  OverviewRulerLane: { Left: 1, Center: 2, Right: 4, Full: 7 },
  // Test helpers
  __test__: { decorationTypes, appliedDecorations },
};
