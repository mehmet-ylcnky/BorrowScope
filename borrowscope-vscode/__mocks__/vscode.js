// Minimal vscode mock for unit testing outside Extension Development Host
module.exports = {
  window: {
    createOutputChannel: (name) => ({
      appendLine: () => {},
      dispose: () => {},
    }),
    showInformationMessage: () => {},
  },
  workspace: {
    getConfiguration: () => ({
      get: (key, defaultValue) => defaultValue,
      update: () => Promise.resolve(),
    }),
  },
  commands: {
    registerCommand: (id, handler) => ({ dispose: () => {} }),
  },
  ConfigurationTarget: { Global: 1, Workspace: 2 },
};
