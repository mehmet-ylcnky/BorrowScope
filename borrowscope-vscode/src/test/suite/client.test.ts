import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("4.2 Language Client Configuration", () => {
  let resolveServerPath: any;

  before(() => {
    const mod = require(path.join(ROOT, "out", "server-path.js"));
    resolveServerPath = mod.resolveServerPath;
  });

  // 1. Returns user-configured path when file exists
  it("uses user-configured path when file exists", () => {
    const tmpBin = path.join(os.tmpdir(), "bs-test-configured-bin");
    fs.writeFileSync(tmpBin, "");
    try {
      const result = resolveServerPath({ extensionPath: "/nonexistent", configuredPath: tmpBin });
      assert.strictEqual(result, tmpBin);
    } finally {
      fs.unlinkSync(tmpBin);
    }
  });

  // 2. Returns bundled path when user path not set
  it("uses bundled binary when user path empty", () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-ext-"));
    const serverDir = path.join(tmpDir, "server");
    fs.mkdirSync(serverDir);
    const bundledBin = path.join(serverDir, "borrowscope-lsp");
    fs.writeFileSync(bundledBin, "");
    try {
      const result = resolveServerPath({ extensionPath: tmpDir, configuredPath: "" });
      assert.strictEqual(result, bundledBin);
    } finally {
      fs.rmSync(tmpDir, { recursive: true });
    }
  });

  // 3. Throws when binary not found anywhere
  it("throws error when binary not found", () => {
    const origPath = process.env.PATH;
    process.env.PATH = "/nonexistent-dir-only";
    try {
      assert.throws(
        () => resolveServerPath({ extensionPath: "/nonexistent", configuredPath: "" }),
        /borrowscope-lsp binary not found/
      );
    } finally {
      process.env.PATH = origPath;
    }
  });

  // 4. Finds binary in system PATH
  it("finds binary in system PATH", () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-path-"));
    const binPath = path.join(tmpDir, "borrowscope-lsp");
    fs.writeFileSync(binPath, "");
    const origPath = process.env.PATH;
    process.env.PATH = tmpDir + path.delimiter + (origPath || "");
    try {
      const result = resolveServerPath({ extensionPath: "/nonexistent", configuredPath: "" });
      assert.strictEqual(result, binPath);
    } finally {
      process.env.PATH = origPath;
      fs.rmSync(tmpDir, { recursive: true });
    }
  });

  // 5. User config takes priority over bundled
  it("user config takes priority over bundled", () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-prio-"));
    const userBin = path.join(tmpDir, "user-lsp");
    const serverDir = path.join(tmpDir, "server");
    fs.mkdirSync(serverDir);
    const bundledBin = path.join(serverDir, "borrowscope-lsp");
    fs.writeFileSync(userBin, "");
    fs.writeFileSync(bundledBin, "");
    try {
      const result = resolveServerPath({ extensionPath: tmpDir, configuredPath: userBin });
      assert.strictEqual(result, userBin);
    } finally {
      fs.rmSync(tmpDir, { recursive: true });
    }
  });

  // 6. Bundled takes priority over PATH
  it("bundled takes priority over PATH", () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-bund-"));
    const serverDir = path.join(tmpDir, "server");
    fs.mkdirSync(serverDir);
    const bundledBin = path.join(serverDir, "borrowscope-lsp");
    fs.writeFileSync(bundledBin, "");
    const pathDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-pathdir-"));
    fs.writeFileSync(path.join(pathDir, "borrowscope-lsp"), "");
    const origPath = process.env.PATH;
    process.env.PATH = pathDir + path.delimiter + (origPath || "");
    try {
      const result = resolveServerPath({ extensionPath: tmpDir, configuredPath: "" });
      assert.strictEqual(result, bundledBin);
    } finally {
      process.env.PATH = origPath;
      fs.rmSync(tmpDir, { recursive: true });
      fs.rmSync(pathDir, { recursive: true });
    }
  });

  // 7. Ignores non-existent user-configured path
  it("ignores non-existent user-configured path", () => {
    const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "bs-noexist-"));
    const serverDir = path.join(tmpDir, "server");
    fs.mkdirSync(serverDir);
    fs.writeFileSync(path.join(serverDir, "borrowscope-lsp"), "");
    try {
      const result = resolveServerPath({ extensionPath: tmpDir, configuredPath: "/no/such/file" });
      assert.strictEqual(result, path.join(serverDir, "borrowscope-lsp"));
    } finally {
      fs.rmSync(tmpDir, { recursive: true });
    }
  });

  // 8. Error message mentions setting name
  it("error message mentions borrowscope.server.path", () => {
    const origPath = process.env.PATH;
    process.env.PATH = "/nonexistent-dir-only";
    try {
      let msg = "";
      try { resolveServerPath({ extensionPath: "/x", configuredPath: "" }); }
      catch (e: any) { msg = e.message; }
      assert.ok(msg.includes("borrowscope.server.path"));
    } finally {
      process.env.PATH = origPath;
    }
  });

  // 9. Error message suggests installing
  it("error message suggests installing", () => {
    const origPath = process.env.PATH;
    process.env.PATH = "/nonexistent-dir-only";
    try {
      let msg = "";
      try { resolveServerPath({ extensionPath: "/x", configuredPath: "" }); }
      catch (e: any) { msg = e.message; }
      assert.ok(msg.includes("Install it"));
    } finally {
      process.env.PATH = origPath;
    }
  });

  // 10. Extension source imports client functions
  it("extension.ts source imports client functions", () => {
    const srcPath = path.join(ROOT, "src", "extension.ts");
    assert.ok(fs.existsSync(srcPath), "extension.ts should exist");
    const content = fs.readFileSync(srcPath, "utf8");
    assert.ok(content.includes("startClient"), "Should import startClient");
    assert.ok(content.includes("stopClient"), "Should import stopClient");
    assert.ok(content.includes("restartClient"), "Should import restartClient");
  });

  // 11. Client passes initialization options to server
  it("client.ts passes initializationOptions", () => {
    const srcPath = path.join(ROOT, "src", "client.ts");
    const content = fs.readFileSync(srcPath, "utf8");
    assert.ok(content.includes("initializationOptions"), "Should pass initializationOptions");
    assert.ok(content.includes("decorations"), "Should include decorations setting");
  });

  // 12. Extension deactivate calls stopClient
  it("deactivate calls stopClient", () => {
    const srcPath = path.join(ROOT, "src", "extension.ts");
    const content = fs.readFileSync(srcPath, "utf8");
    assert.ok(content.includes("async function deactivate"), "deactivate should be async");
    assert.ok(content.includes("stopClient()"), "deactivate should call stopClient");
  });

  // 13. Client uses vscode-languageclient (auto crash recovery)
  it("client uses LanguageClient with crash recovery", () => {
    const srcPath = path.join(ROOT, "src", "client.ts");
    const content = fs.readFileSync(srcPath, "utf8");
    assert.ok(content.includes("LanguageClient"), "Should use LanguageClient class");
    assert.ok(content.includes("TransportKind.stdio"), "Should use stdio transport");
  });
});
