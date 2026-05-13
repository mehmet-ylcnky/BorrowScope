// Register vscode mock before tests run
const Module = require("module");
const path = require("path");
const origResolve = Module._resolveFilename;
Module._resolveFilename = function (request, parent, ...args) {
  if (request === "vscode") {
    return path.resolve(__dirname, "__mocks__", "vscode.js");
  }
  return origResolve.call(this, request, parent, ...args);
};
