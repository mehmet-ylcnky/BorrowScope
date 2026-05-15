import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("7.3 Reference Count History", () => {
  let refcount: any;
  let panelSrc: string;

  before(() => {
    refcount = require(path.join(ROOT, "out", "graph", "refcount.js"));
    panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
  });

  // 1. Single Rc with no clones: count 1 then drops to 0
  it("single Rc produces series with new(1) and drop(0)", () => {
    const graph = { function_name: "f", end_line: 10, variables: [{ name: "rc1", line: 2, type_display: "Rc<i32>", ownership_category: "Rc" }], borrow_scopes: [], rc_clones: [], conflicts: [] };
    const result = refcount.buildRefCountData(graph);
    assert.strictEqual(result.length, 1);
    assert.strictEqual(result[0].name, "rc1");
    assert.strictEqual(result[0].events[0].count, 1);
    assert.strictEqual(result[0].events[result[0].events.length - 1].count, 0);
  });

  // 2. Rc with 2 clones: steps up to 3
  it("Rc with 2 clones reaches count 3", () => {
    const graph = { function_name: "f", end_line: 15, variables: [{ name: "rc1", line: 2, type_display: "Rc<i32>", ownership_category: "Rc" }, { name: "rc2", line: 3, type_display: "Rc<i32>", ownership_category: "Rc" }, { name: "rc3", line: 4, type_display: "Rc<i32>", ownership_category: "Rc" }], borrow_scopes: [], rc_clones: [{ clone_variable: "rc2", source_variable: "rc1", clone_type: "Rc", line: 3 }, { clone_variable: "rc3", source_variable: "rc1", clone_type: "Rc", line: 4 }], conflicts: [] };
    const result = refcount.buildRefCountData(graph);
    assert.strictEqual(result.length, 1);
    const maxCount = Math.max(...result[0].events.map((e: any) => e.count));
    assert.strictEqual(maxCount, 3);
  });

  // 3. No Rc/Arc produces empty result
  it("no Rc/Arc produces empty series", () => {
    const graph = { function_name: "f", end_line: 10, variables: [{ name: "x", line: 2, type_display: "i32", ownership_category: "Copy" }], borrow_scopes: [], rc_clones: [], conflicts: [] };
    const result = refcount.buildRefCountData(graph);
    assert.strictEqual(result.length, 0);
  });

  // 4. Multiple Rc families produce separate series
  it("multiple Rc sources produce separate series", () => {
    const graph = { function_name: "f", end_line: 10, variables: [{ name: "a", line: 2, type_display: "Rc<i32>", ownership_category: "Rc" }, { name: "b", line: 3, type_display: "Rc<String>", ownership_category: "Rc" }], borrow_scopes: [], rc_clones: [], conflicts: [] };
    const result = refcount.buildRefCountData(graph);
    assert.strictEqual(result.length, 2);
  });

  // 5. Events have correct action labels
  it("events have action labels (new, clone, drop)", () => {
    const graph = { function_name: "f", end_line: 10, variables: [{ name: "rc1", line: 2, type_display: "Rc<i32>", ownership_category: "Rc" }, { name: "rc2", line: 4, type_display: "Rc<i32>", ownership_category: "Rc" }], borrow_scopes: [], rc_clones: [{ clone_variable: "rc2", source_variable: "rc1", clone_type: "Rc", line: 4 }], conflicts: [] };
    const result = refcount.buildRefCountData(graph);
    const actions = result[0].events.map((e: any) => e.action);
    assert.ok(actions.includes("new"));
    assert.ok(actions.includes("clone"));
    assert.ok(actions.includes("drop"));
  });

  // 6. Panel has refcount-container
  it("panel has refcount container", () => {
    assert.ok(panelSrc.includes('id="refcount-container"'));
  });

  // 7. Panel has RefCount view button
  it("panel has RefCount view button", () => {
    assert.ok(panelSrc.includes('data-view="refcount"'));
  });

  // 8. renderRefCount function exists
  it("renderRefCount function exists", () => {
    assert.ok(panelSrc.includes("function renderRefCount()"));
  });

  // 9. Chart uses step curve
  it("chart uses step curve for discrete count changes", () => {
    assert.ok(panelSrc.includes("curveStepAfter"));
  });

  // 10. Shows leak warning when count > 0 at end
  it("shows leak warning for non-zero final count", () => {
    assert.ok(panelSrc.includes("Potential leak"));
  });
});
