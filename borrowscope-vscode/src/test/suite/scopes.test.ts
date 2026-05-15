import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("7.2 Scope Nesting View", () => {
  let scopes: any;
  let panelSrc: string;

  before(() => {
    scopes = require(path.join(ROOT, "out", "graph", "scopes.js"));
    panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
  });

  // 1. buildScopeData returns root scope
  it("buildScopeData returns root function scope", () => {
    const graph = { function_name: "test", start_line: 1, end_line: 10, variables: [{ name: "x", line: 2, type_display: "i32", ownership_category: "Copy" }], borrow_scopes: [], conflicts: [] };
    const result = scopes.buildScopeData(graph);
    assert.strictEqual(result.root.kind, "function");
    assert.strictEqual(result.root.name, "test");
  });

  // 2. Variables assigned to root scope
  it("variables appear in root scope", () => {
    const graph = { function_name: "f", start_line: 1, end_line: 10, variables: [{ name: "a", line: 2, type_display: "i32", ownership_category: "Copy" }, { name: "b", line: 3, type_display: "String", ownership_category: "Owned" }], borrow_scopes: [], conflicts: [] };
    const result = scopes.buildScopeData(graph);
    assert.strictEqual(result.root.variables.length, 2);
  });

  // 3. Inner block detected from borrow scopes
  it("detects inner block from short-lived borrows", () => {
    const graph = { function_name: "f", start_line: 1, end_line: 20, variables: [{ name: "data", line: 2, type_display: "Vec", ownership_category: "Owned" }, { name: "r", line: 5, type_display: "&Vec", ownership_category: "SharedRef" }], borrow_scopes: [{ borrower_name: "r", target_name: "data", is_mutable: false, start_line: 5, end_line: 8 }], conflicts: [] };
    const result = scopes.buildScopeData(graph);
    assert.ok(result.root.children.length > 0, "Should detect inner block");
    assert.strictEqual(result.root.children[0].kind, "block");
  });

  // 4. Drop order is reverse of declaration
  it("drop order is reverse declaration order", () => {
    const graph = { function_name: "f", start_line: 1, end_line: 10, variables: [{ name: "first", line: 2, type_display: "i32", ownership_category: "Copy" }, { name: "second", line: 3, type_display: "i32", ownership_category: "Copy" }, { name: "third", line: 4, type_display: "i32", ownership_category: "Copy" }], borrow_scopes: [], conflicts: [] };
    const result = scopes.buildScopeData(graph);
    assert.deepStrictEqual(result.dropOrder, ["third", "second", "first"]);
  });

  // 5. Empty function produces empty scope
  it("empty function has no variables", () => {
    const graph = { function_name: "empty", start_line: 1, end_line: 2, variables: [], borrow_scopes: [], conflicts: [] };
    const result = scopes.buildScopeData(graph);
    assert.strictEqual(result.root.variables.length, 0);
    assert.strictEqual(result.root.children.length, 0);
  });

  // 6. Panel has scopes-container div
  it("panel HTML has scopes container", () => {
    assert.ok(panelSrc.includes('id="scopes-container"'));
  });

  // 7. Panel has Scopes view button
  it("panel has Scopes view button", () => {
    assert.ok(panelSrc.includes('data-view="scopes"'));
  });

  // 8. renderScopes function exists
  it("renderScopes function exists in panel", () => {
    assert.ok(panelSrc.includes("function renderScopes()"));
  });

  // 9. Scopes view shows drop order
  it("scopes view displays drop order", () => {
    assert.ok(panelSrc.includes("Drop order"));
    assert.ok(panelSrc.includes("reverse()"));
  });

  // 10. Scopes view has click-to-navigate
  it("scopes view supports click to navigate", () => {
    assert.ok(panelSrc.includes("data-line"));
    assert.ok(panelSrc.includes("nodeClicked"));
  });

  // 11. Scopes view links to table on hover
  it("scopes view highlights table row on hover", () => {
    assert.ok(panelSrc.includes("scope-var"));
    assert.ok(panelSrc.includes("data-var"));
  });

  // 12. Scope variables show category color
  it("scope variables are color-coded by category", () => {
    assert.ok(panelSrc.includes("ownership_category"));
    assert.ok(panelSrc.includes("border-left:3px solid"));
  });
});
