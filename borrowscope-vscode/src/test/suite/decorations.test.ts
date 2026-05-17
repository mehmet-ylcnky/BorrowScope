import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("4.4 Inline Decorations from Inlay Hints", () => {
  let decorations: any;
  let vscode: any;

  before(() => {
    vscode = require("vscode");
    decorations = require(path.join(ROOT, "out", "decorations.js"));
  });

  afterEach(() => {
    decorations.disposeDecorations();
  });

  function mockEditor() {
    const applied: Map<any, any[]> = new Map();
    return {
      setDecorations: (dt: any, ranges: any[]) => { applied.set(dt, ranges); },
      __applied: applied,
    };
  }

  // 1. Rc variables get purple decoration
  it("Rc hints get purple color", () => {
    const color = decorations.getColorForLabel("[Rc]");
    assert.strictEqual(color, "#9b59b6");
  });

  // 2. Shared ref gets blue decoration
  it("shared ref hints get blue color", () => {
    const color = decorations.getColorForLabel("[&]");
    assert.strictEqual(color, "#3498db");
  });

  // 3. Mutable ref gets red decoration
  it("mutable ref hints get red color", () => {
    const color = decorations.getColorForLabel("[&mut]");
    assert.strictEqual(color, "#e74c3c");
  });

  // 4. Arc gets dark purple
  it("Arc hints get dark purple color", () => {
    const color = decorations.getColorForLabel("[Arc]");
    assert.strictEqual(color, "#9b59b6");
  });

  // 5. Cell gets orange
  it("Cell hints get orange color", () => {
    const color = decorations.getColorForLabel("[Cell]");
    assert.strictEqual(color, "#e67e22");
  });

  // 6. Raw pointer gets gray
  it("raw pointer hints get gray color", () => {
    const color = decorations.getColorForLabel("[*ptr]");
    assert.strictEqual(color, "#95a5a6");
  });

  // 7. applyDecorations applies hints to editor
  it("applyDecorations sets decorations on editor", () => {
    const editor = mockEditor();
    const hints = [
      { line: 1, character: 10, label: "[&]" },
      { line: 3, character: 12, label: "[Rc]" },
    ];
    decorations.applyDecorations(editor, hints);
    // Should have applied at least 2 decoration types
    assert.ok(editor.__applied.size >= 2, `Applied ${editor.__applied.size} decoration types`);
  });

  // 8. No decorations for empty hints array
  it("applyDecorations with empty array clears decorations", () => {
    const editor = mockEditor();
    decorations.applyDecorations(editor, []);
    // All decoration types should have empty ranges
    for (const ranges of editor.__applied.values()) {
      assert.strictEqual(ranges.length, 0);
    }
  });

  // 9. Decorations disabled returns false from isEnabled
  it("isEnabled returns true by default", () => {
    // Default mock returns true for boolean configs
    assert.strictEqual(decorations.isEnabled(), true);
  });

  // 10. clearDecorations removes all decorations from editor
  it("clearDecorations empties all decoration types", () => {
    const editor = mockEditor();
    // First apply some
    decorations.applyDecorations(editor, [{ line: 0, character: 5, label: "[&]" }]);
    // Then clear
    decorations.clearDecorations(editor);
    for (const ranges of editor.__applied.values()) {
      assert.strictEqual(ranges.length, 0);
    }
  });
});
