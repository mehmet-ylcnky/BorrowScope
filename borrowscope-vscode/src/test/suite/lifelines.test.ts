import * as assert from "assert";
import * as path from "path";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("4.5 Gutter Icons and Lifecycle Flow Lines", () => {
  let lifelines: any;
  let vscode: any;

  before(() => {
    vscode = require("vscode");
    lifelines = require(path.join(ROOT, "out", "lifelines.js"));
  });

  afterEach(() => {
    lifelines.disposeLifelines();
  });

  function mockEditor() {
    const applied: Map<any, any[]> = new Map();
    return {
      setDecorations: (dt: any, ranges: any[]) => { applied.set(dt, ranges); },
      __applied: applied,
    };
  }

  // 1. Shared borrow gets blue flow lines
  it("shared borrow produces blue flow characters", () => {
    const editor = mockEditor();
    const scopes = [{ borrower: "r", target: "data", is_mutable: false, range: { start: { line: 2 }, end: { line: 4 } } }];
    lifelines.applyLifelines(editor, scopes);
    // Should have applied decorations
    assert.ok(editor.__applied.size > 0, "Should apply decorations");
    // Check one of the decoration options has blue color
    let foundBlue = false;
    for (const decs of editor.__applied.values()) {
      for (const d of decs) {
        if (d.renderOptions?.before?.color === "#3498db") foundBlue = true;
      }
    }
    assert.ok(foundBlue, "Should have blue decorations for shared borrow");
  });

  // 2. Mutable borrow gets red flow lines
  it("mutable borrow produces red flow characters", () => {
    const editor = mockEditor();
    const scopes = [{ borrower: "m", target: "data", is_mutable: true, range: { start: { line: 5 }, end: { line: 7 } } }];
    lifelines.applyLifelines(editor, scopes);
    let foundRed = false;
    for (const decs of editor.__applied.values()) {
      for (const d of decs) {
        if (d.renderOptions?.before?.color === "#e74c3c") foundRed = true;
      }
    }
    assert.ok(foundRed, "Should have red decorations for mutable borrow");
  });

  // 3. Start line gets ├─ character
  it("start line gets branch start character", () => {
    const editor = mockEditor();
    const scopes = [{ borrower: "r", target: "x", is_mutable: false, range: { start: { line: 3 }, end: { line: 5 } } }];
    lifelines.applyLifelines(editor, scopes);
    let startChar = "";
    for (const decs of editor.__applied.values()) {
      for (const d of decs) {
        if (d.range.start.line === 3) startChar = d.renderOptions?.before?.contentText || "";
      }
    }
    assert.strictEqual(startChar, "├─", `Start should be ├─, got: ${startChar}`);
  });

  // 4. End line gets ╰─ character
  it("end line gets branch end character", () => {
    const editor = mockEditor();
    const scopes = [{ borrower: "r", target: "x", is_mutable: false, range: { start: { line: 3 }, end: { line: 5 } } }];
    lifelines.applyLifelines(editor, scopes);
    let endChar = "";
    for (const decs of editor.__applied.values()) {
      for (const d of decs) {
        if (d.range.start.line === 5) endChar = d.renderOptions?.before?.contentText || "";
      }
    }
    assert.strictEqual(endChar, "╰─", `End should be ╰─, got: ${endChar}`);
  });

  // 5. Middle lines get │ character
  it("middle lines get continuation character", () => {
    const editor = mockEditor();
    const scopes = [{ borrower: "r", target: "x", is_mutable: false, range: { start: { line: 3 }, end: { line: 6 } } }];
    lifelines.applyLifelines(editor, scopes);
    let midChar = "";
    for (const decs of editor.__applied.values()) {
      for (const d of decs) {
        if (d.range.start.line === 4) midChar = d.renderOptions?.before?.contentText || "";
      }
    }
    assert.strictEqual(midChar, "│ ", `Middle should be │ , got: "${midChar}"`);
  });

  // 6. Hover message on start line mentions borrow starts
  it("start line hover mentions borrow starts", () => {
    const editor = mockEditor();
    const scopes = [{ borrower: "r", target: "data", is_mutable: false, range: { start: { line: 2 }, end: { line: 4 } } }];
    lifelines.applyLifelines(editor, scopes);
    let hover = "";
    for (const decs of editor.__applied.values()) {
      for (const d of decs) {
        if (d.range.start.line === 2) hover = d.hoverMessage || "";
      }
    }
    assert.ok(hover.includes("starts"), `Hover should mention starts: ${hover}`);
    assert.ok(hover.includes("r"), `Hover should mention borrower: ${hover}`);
  });

  // 7. Hover message on end line mentions borrow ends
  it("end line hover mentions borrow ends", () => {
    const editor = mockEditor();
    const scopes = [{ borrower: "r", target: "data", is_mutable: false, range: { start: { line: 2 }, end: { line: 4 } } }];
    lifelines.applyLifelines(editor, scopes);
    let hover = "";
    for (const decs of editor.__applied.values()) {
      for (const d of decs) {
        if (d.range.start.line === 4) hover = d.hoverMessage || "";
      }
    }
    assert.ok(hover.includes("ends"), `Hover should mention ends: ${hover}`);
  });

  // 8. Empty scopes produces no decorations
  it("empty scopes array produces no decorations", () => {
    const editor = mockEditor();
    lifelines.applyLifelines(editor, []);
    // All applied should be empty
    for (const decs of editor.__applied.values()) {
      assert.strictEqual(decs.length, 0);
    }
  });

  // 9. clearLifelines removes all decorations
  it("clearLifelines empties all decoration types", () => {
    const editor = mockEditor();
    lifelines.applyLifelines(editor, [{ borrower: "r", target: "x", is_mutable: false, range: { start: { line: 0 }, end: { line: 2 } } }]);
    lifelines.clearLifelines(editor);
    for (const decs of editor.__applied.values()) {
      assert.strictEqual(decs.length, 0);
    }
  });

  // 10. isLifelinesEnabled returns true by default
  it("isLifelinesEnabled returns true by default", () => {
    assert.strictEqual(lifelines.isLifelinesEnabled(), true);
  });
});
