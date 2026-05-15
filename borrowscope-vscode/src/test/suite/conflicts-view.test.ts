import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("7.5 Conflict Highlight Mode", () => {
  let panelSrc: string;

  before(() => {
    panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
  });

  // 1. Panel has conflicts-container
  it("panel has conflicts container", () => {
    assert.ok(panelSrc.includes('id="conflicts-container"'));
  });

  // 2. Panel has Conflicts view button
  it("panel has Conflicts view button", () => {
    assert.ok(panelSrc.includes('data-view="conflicts"'));
  });

  // 3. renderConflicts function exists
  it("renderConflicts function exists", () => {
    assert.ok(panelSrc.includes("function renderConflicts()"));
  });

  // 4. Shows no-conflicts message when empty
  it("shows no conflicts message when none detected", () => {
    assert.ok(panelSrc.includes("No conflicts detected"));
  });

  // 5. Shows conflict count
  it("shows conflict count", () => {
    assert.ok(panelSrc.includes("conflict(s) detected"));
  });

  // 6. Shows both borrow names
  it("shows borrow_a and borrow_b names", () => {
    assert.ok(panelSrc.includes("c.borrow_a"));
    assert.ok(panelSrc.includes("c.borrow_b"));
  });

  // 7. Shows overlap line range
  it("shows overlap line range", () => {
    assert.ok(panelSrc.includes("overlap_start_line"));
    assert.ok(panelSrc.includes("overlap_end_line"));
  });

  // 8. Shows variable name involved in conflict
  it("shows variable name in conflict", () => {
    assert.ok(panelSrc.includes("c.variable"));
  });

  // 9. Has clickable link to navigate to conflict
  it("has clickable link to navigate to conflict line", () => {
    assert.ok(panelSrc.includes("Go to line"));
    assert.ok(panelSrc.includes("nodeClicked"));
  });

  // 10. Uses red styling for conflicts
  it("uses red styling for conflict indicators", () => {
    assert.ok(panelSrc.includes("#f85149"));
    assert.ok(panelSrc.includes("border:2px solid #f85149"));
  });
});
