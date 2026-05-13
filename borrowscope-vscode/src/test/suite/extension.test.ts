import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("4.1 Extension Project Setup", () => {
  // 1. package.json exists and is valid JSON
  it("package.json is valid JSON", () => {
    const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    assert.strictEqual(pkg.name, "borrowscope");
  });

  // 2. Activation event is onLanguage:rust
  it("activates on Rust files", () => {
    const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    assert.ok(pkg.activationEvents.includes("onLanguage:rust"));
  });

  // 3. All 4 commands are registered
  it("registers 4 commands", () => {
    const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    const commands = pkg.contributes.commands.map((c: any) => c.command);
    assert.ok(commands.includes("borrowscope.showGraph"));
    assert.ok(commands.includes("borrowscope.inspectVariable"));
    assert.ok(commands.includes("borrowscope.toggleDecorations"));
    assert.ok(commands.includes("borrowscope.restartServer"));
  });

  // 4. Configuration properties exist
  it("defines configuration properties", () => {
    const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    const props = pkg.contributes.configuration.properties;
    assert.ok("borrowscope.server.path" in props);
    assert.ok("borrowscope.decorations.enabled" in props);
    assert.ok("borrowscope.decorations.borrowScopes" in props);
    assert.ok("borrowscope.graph.layout" in props);
    assert.strictEqual(props["borrowscope.graph.layout"].default, "hierarchical");
  });

  // 5. Main entry point is set
  it("main points to out/extension.js", () => {
    const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    assert.strictEqual(pkg.main, "./out/extension.js");
  });

  // 6. Build output exists
  it("build output exists", () => {
    assert.ok(fs.existsSync(path.join(ROOT, "out", "extension.js")));
  });

  // 7. Extension exports activate and deactivate
  it("extension exports activate and deactivate", () => {
    const ext = require(path.join(ROOT, "out", "extension.js"));
    assert.strictEqual(typeof ext.activate, "function");
    assert.strictEqual(typeof ext.deactivate, "function");
  });

  // 8. vscode-languageclient is a dependency
  it("depends on vscode-languageclient", () => {
    const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    assert.ok("vscode-languageclient" in pkg.dependencies);
  });

  // 9. Engine requires vscode ^1.85.0
  it("requires vscode ^1.85.0", () => {
    const pkg = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
    assert.strictEqual(pkg.engines.vscode, "^1.85.0");
  });

  // 10. TypeScript source compiles without errors (build succeeded)
  it("esbuild config exists and is valid", () => {
    const esbuildPath = path.join(ROOT, "esbuild.js");
    assert.ok(fs.existsSync(esbuildPath));
    const content = fs.readFileSync(esbuildPath, "utf8");
    assert.ok(content.includes("entryPoints"));
    assert.ok(content.includes("extension.ts"));
  });
});
