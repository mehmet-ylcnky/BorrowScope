import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("7.6 Comparison View", () => {
  let comparison: any;
  let panelSrc: string;

  before(() => {
    comparison = require(path.join(ROOT, "out", "graph", "comparison.js"));
    panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
  });

  // 1. Identical graphs produce no changes
  it("identical graphs produce no changes", () => {
    const graph = { variables: [{ name: "x" }], borrow_scopes: [], moves: [] };
    const result = comparison.computeOwnershipDiff(graph, graph);
    assert.strictEqual(result.hasChanges, false);
    assert.strictEqual(result.summary, "No ownership changes");
  });

  // 2. Added variable detected
  it("detects added variable", () => {
    const before = { variables: [{ name: "a" }], borrow_scopes: [], moves: [] };
    const after = { variables: [{ name: "a" }, { name: "b" }], borrow_scopes: [], moves: [] };
    const result = comparison.computeOwnershipDiff(before, after);
    assert.deepStrictEqual(result.addedVariables, ["b"]);
    assert.strictEqual(result.hasChanges, true);
  });

  // 3. Removed variable detected
  it("detects removed variable", () => {
    const before = { variables: [{ name: "a" }, { name: "b" }], borrow_scopes: [], moves: [] };
    const after = { variables: [{ name: "a" }], borrow_scopes: [], moves: [] };
    const result = comparison.computeOwnershipDiff(before, after);
    assert.deepStrictEqual(result.removedVariables, ["b"]);
  });

  // 4. Added borrow detected
  it("detects added borrow", () => {
    const before = { variables: [], borrow_scopes: [], moves: [] };
    const after = { variables: [], borrow_scopes: [{ borrower_name: "r", target_name: "data" }], moves: [] };
    const result = comparison.computeOwnershipDiff(before, after);
    assert.strictEqual(result.addedBorrows.length, 1);
  });

  // 5. Removed borrow detected
  it("detects removed borrow", () => {
    const before = { variables: [], borrow_scopes: [{ borrower_name: "r", target_name: "data" }], moves: [] };
    const after = { variables: [], borrow_scopes: [], moves: [] };
    const result = comparison.computeOwnershipDiff(before, after);
    assert.strictEqual(result.removedBorrows.length, 1);
  });

  // 6. Summary format correct
  it("summary shows correct format", () => {
    const before = { variables: [{ name: "a" }], borrow_scopes: [], moves: [] };
    const after = { variables: [{ name: "a" }, { name: "b" }, { name: "c" }], borrow_scopes: [{ borrower_name: "r", target_name: "a" }], moves: [] };
    const result = comparison.computeOwnershipDiff(before, after);
    assert.ok(result.summary.includes("+2 vars"));
    assert.ok(result.summary.includes("+1 borrows"));
  });

  // 7. Panel has compare-container
  it("panel has compare container", () => {
    assert.ok(panelSrc.includes('id="compare-container"'));
  });

  // 8. Panel has Compare view button
  it("panel has Compare view button", () => {
    assert.ok(panelSrc.includes('data-view="compare"'));
  });

  // 9. renderCompare function exists
  it("renderCompare function exists", () => {
    assert.ok(panelSrc.includes("function renderCompare()"));
  });

  // 10. Shows "No ownership changes" for identical
  it("shows no changes message", () => {
    assert.ok(panelSrc.includes("No ownership changes"));
  });

  // 11. Shows added variables in green
  it("shows added variables with green indicator", () => {
    assert.ok(panelSrc.includes("Added variables"));
    assert.ok(panelSrc.includes("#3fb950"));
  });

  // 12. Shows removed variables in red
  it("shows removed variables with red strikethrough", () => {
    assert.ok(panelSrc.includes("Removed variables"));
    assert.ok(panelSrc.includes("line-through"));
  });
});
