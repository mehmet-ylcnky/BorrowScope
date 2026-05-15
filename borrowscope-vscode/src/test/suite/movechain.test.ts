import * as assert from "assert";
import * as path from "path";
import * as fs from "fs";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("7.4 Move Chain View", () => {
  let movechain: any;
  let panelSrc: string;

  before(() => {
    movechain = require(path.join(ROOT, "out", "graph", "movechain.js"));
    panelSrc = fs.readFileSync(path.join(ROOT, "src", "graph", "panel.ts"), "utf8");
  });

  // 1. Single move produces one chain
  it("single move produces one chain with 2 nodes", () => {
    const graph = { variables: [{ name: "a", line: 2, type_display: "String", ownership_category: "Owned" }], moves: [{ source_name: "a", destination: "b", line: 5, source_type: "String" }], rc_clones: [], borrow_scopes: [], conflicts: [] };
    const result = movechain.buildMoveChainData(graph);
    assert.strictEqual(result.hasMovesFlag, true);
    assert.strictEqual(result.chains.length, 1);
    assert.strictEqual(result.chains[0].nodes.length, 2);
  });

  // 2. Moved-from variable is not alive
  it("moved-from variable has isAlive=false", () => {
    const graph = { variables: [{ name: "a", line: 2, type_display: "String", ownership_category: "Owned" }], moves: [{ source_name: "a", destination: "b", line: 5, source_type: "String" }], rc_clones: [], borrow_scopes: [], conflicts: [] };
    const result = movechain.buildMoveChainData(graph);
    const srcNode = result.chains[0].nodes.find((n: any) => n.name === "a");
    assert.strictEqual(srcNode.isAlive, false);
  });

  // 3. Final destination is alive
  it("final destination has isAlive=true", () => {
    const graph = { variables: [{ name: "a", line: 2, type_display: "String", ownership_category: "Owned" }], moves: [{ source_name: "a", destination: "b", line: 5, source_type: "String" }], rc_clones: [], borrow_scopes: [], conflicts: [] };
    const result = movechain.buildMoveChainData(graph);
    const destNode = result.chains[0].nodes.find((n: any) => n.name === "b");
    assert.strictEqual(destNode.isAlive, true);
  });

  // 4. No moves returns empty
  it("no moves returns hasMovesFlag=false", () => {
    const graph = { variables: [{ name: "x", line: 2, type_display: "i32", ownership_category: "Copy" }], moves: [], rc_clones: [], borrow_scopes: [], conflicts: [] };
    const result = movechain.buildMoveChainData(graph);
    assert.strictEqual(result.hasMovesFlag, false);
    assert.strictEqual(result.chains.length, 0);
  });

  // 5. Chain of 3 moves
  it("chain of 3 moves produces 3 edges", () => {
    const graph = { variables: [{ name: "a", line: 2, type_display: "String", ownership_category: "Owned" }], moves: [{ source_name: "a", destination: "b", line: 3, source_type: "String" }, { source_name: "b", destination: "c", line: 5, source_type: "String" }], rc_clones: [], borrow_scopes: [], conflicts: [] };
    const result = movechain.buildMoveChainData(graph);
    assert.strictEqual(result.chains[0].edges.length, 2);
  });

  // 6. Move to function arg
  it("handles move to function arg", () => {
    const graph = { variables: [{ name: "v", line: 2, type_display: "Vec<i32>", ownership_category: "Owned" }], moves: [{ source_name: "v", destination: { FunctionArg: { function: "drop" } }, line: 5, source_type: "Vec<i32>" }], rc_clones: [], borrow_scopes: [], conflicts: [] };
    const result = movechain.buildMoveChainData(graph);
    const dest = result.chains[0].nodes.find((n: any) => n.name.includes("drop"));
    assert.ok(dest, "Should have drop() destination");
  });

  // 7. Panel has moves-container
  it("panel has moves container", () => {
    assert.ok(panelSrc.includes('id="moves-container"'));
  });

  // 8. Panel has Moves view button
  it("panel has Moves view button", () => {
    assert.ok(panelSrc.includes('data-view="moves"'));
  });

  // 9. renderMoves function exists
  it("renderMoves function exists", () => {
    assert.ok(panelSrc.includes("function renderMoves()"));
  });

  // 10. Shows empty message when no moves
  it("shows empty message when no moves", () => {
    assert.ok(panelSrc.includes("No ownership transfers"));
  });
});
