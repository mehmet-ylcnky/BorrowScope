import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("8.2 Keyboard Shortcuts", () => {
  let pkgJson: any;

  before(() => {
    pkgJson = JSON.parse(fs.readFileSync(path.join(ROOT, "package.json"), "utf8"));
  });

  const keybindings = () => pkgJson.contributes.keybindings;
  const commands = () => pkgJson.contributes.commands;

  // === Keybindings exist ===
  it("has keybindings section", () => {
    assert.ok(Array.isArray(keybindings()));
    assert.ok(keybindings().length >= 6);
  });

  it("Ctrl+Shift+O opens graph", () => {
    const kb = keybindings().find((k: any) => k.command === "borrowscope.showGraph");
    assert.ok(kb);
    assert.strictEqual(kb.key, "ctrl+shift+o");
    assert.strictEqual(kb.when, "editorLangId == rust");
  });

  it("Ctrl+Shift+I inspects variable", () => {
    const kb = keybindings().find((k: any) => k.command === "borrowscope.inspectVariable");
    assert.ok(kb);
    assert.strictEqual(kb.key, "ctrl+shift+i");
    assert.strictEqual(kb.when, "editorLangId == rust");
  });

  it("Ctrl+Shift+D toggles decorations", () => {
    const kb = keybindings().find((k: any) => k.command === "borrowscope.toggleDecorations");
    assert.ok(kb);
    assert.strictEqual(kb.key, "ctrl+shift+d");
    assert.strictEqual(kb.when, "editorLangId == rust");
  });

  it("Alt+Shift+N goes to next conflict", () => {
    const kb = keybindings().find((k: any) => k.command === "borrowscope.nextConflict");
    assert.ok(kb);
    assert.strictEqual(kb.key, "alt+shift+n");
    assert.strictEqual(kb.when, "editorLangId == rust");
  });

  it("Alt+Shift+P goes to previous conflict", () => {
    const kb = keybindings().find((k: any) => k.command === "borrowscope.prevConflict");
    assert.ok(kb);
    assert.strictEqual(kb.key, "alt+shift+p");
    assert.strictEqual(kb.when, "editorLangId == rust");
  });

  it("Ctrl+Shift+G focuses graph panel", () => {
    const kb = keybindings().find((k: any) => k.command === "borrowscope.focusGraph");
    assert.ok(kb);
    assert.strictEqual(kb.key, "ctrl+shift+g");
    assert.strictEqual(kb.when, "editorLangId == rust");
  });

  // === All keybindings have 'when' clause for Rust ===
  it("all keybindings are scoped to Rust files", () => {
    for (const kb of keybindings()) {
      assert.ok(kb.when && kb.when.includes("rust"), `${kb.command} missing Rust 'when' clause`);
    }
  });

  // === Commands exist for all keybindings ===
  it("all keybinding commands are registered", () => {
    for (const kb of keybindings()) {
      const cmd = commands().find((c: any) => c.command === kb.command);
      assert.ok(cmd, `Command ${kb.command} not found in commands list`);
    }
  });

  // === New commands exist ===
  it("nextConflict command exists", () => {
    assert.ok(commands().some((c: any) => c.command === "borrowscope.nextConflict"));
  });

  it("prevConflict command exists", () => {
    assert.ok(commands().some((c: any) => c.command === "borrowscope.prevConflict"));
  });

  it("focusGraph command exists", () => {
    assert.ok(commands().some((c: any) => c.command === "borrowscope.focusGraph"));
  });

  // === Implementation exists ===
  it("extension.ts has nextConflict implementation", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    assert.ok(src.includes("function nextConflict"));
    assert.ok(src.includes("getDiagnostics"));
  });

  it("extension.ts has prevConflict implementation", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    assert.ok(src.includes("function prevConflict"));
  });

  it("extension.ts has focusGraph implementation", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    assert.ok(src.includes("function focusGraph"));
  });

  it("nextConflict wraps around to first", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    assert.ok(src.includes("Wrap around to first"));
  });

  it("prevConflict wraps around to last", () => {
    const src = fs.readFileSync(path.join(ROOT, "src", "extension.ts"), "utf8");
    assert.ok(src.includes("Wrap around to last"));
  });
});
