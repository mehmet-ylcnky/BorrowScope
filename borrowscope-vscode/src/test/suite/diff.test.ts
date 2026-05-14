import * as assert from "assert";
import * as path from "path";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("6.4 Diff-Based UI Updates", () => {
  let diff: any;

  before(() => {
    diff = require(path.join(ROOT, "out", "graph", "diff.js"));
  });

  function makeModel(nodes: any[], edges: any[] = []): any {
    return {
      nodes: nodes.map((n) => ({ id: n.id, name: n.id, type: n.type || "i32", category: n.category || "Owned", line: n.line || 1, isCopy: false, isAlive: n.isAlive !== false, size: n.size || 1 })),
      edges: edges.map((e, i) => ({ id: e.id || `e${i}`, source: e.source, target: e.target, kind: e.kind || "shared_borrow", isMutable: false, isActive: true, label: "&" })),
      metadata: { functionName: "test", file: "", totalNodes: nodes.length, totalEdges: edges.length, hasConflicts: false, stats: { variables: nodes.length, borrows: 0, moves: 0, conflicts: 0 } },
    };
  }

  // 1. No old model — all nodes/edges are "added"
  it("no old model treats everything as added", () => {
    const newModel = makeModel([{ id: "a" }, { id: "b" }], [{ id: "e0", source: "a", target: "b" }]);
    const result = diff.computeGraphDiff(undefined, newModel);
    assert.strictEqual(result.addedNodes.length, 2);
    assert.strictEqual(result.addedEdges.length, 1);
    assert.strictEqual(result.removedNodeIds.length, 0);
    assert.strictEqual(result.hasChanges, true);
  });

  // 2. Identical models — no changes
  it("identical models produce no diff", () => {
    const model = makeModel([{ id: "x" }]);
    const result = diff.computeGraphDiff(model, model);
    assert.strictEqual(result.hasChanges, false);
    assert.strictEqual(result.addedNodes.length, 0);
    assert.strictEqual(result.removedNodeIds.length, 0);
    assert.strictEqual(result.modifiedNodes.length, 0);
  });

  // 3. Added node detected
  it("detects added node", () => {
    const old = makeModel([{ id: "a" }]);
    const newM = makeModel([{ id: "a" }, { id: "b" }]);
    const result = diff.computeGraphDiff(old, newM);
    assert.strictEqual(result.addedNodes.length, 1);
    assert.strictEqual(result.addedNodes[0].id, "b");
    assert.strictEqual(result.hasChanges, true);
  });

  // 4. Removed node detected
  it("detects removed node", () => {
    const old = makeModel([{ id: "a" }, { id: "b" }]);
    const newM = makeModel([{ id: "a" }]);
    const result = diff.computeGraphDiff(old, newM);
    assert.strictEqual(result.removedNodeIds.length, 1);
    assert.strictEqual(result.removedNodeIds[0], "b");
  });

  // 5. Modified node category detected
  it("detects category change", () => {
    const old = makeModel([{ id: "x", category: "Owned" }]);
    const newM = makeModel([{ id: "x", category: "SharedRef" }]);
    const result = diff.computeGraphDiff(old, newM);
    assert.strictEqual(result.modifiedNodes.length, 1);
    assert.strictEqual(result.modifiedNodes[0].id, "x");
    assert.strictEqual(result.modifiedNodes[0].changes.category, "SharedRef");
  });

  // 6. Modified node isAlive detected
  it("detects isAlive change (moved variable)", () => {
    const old = makeModel([{ id: "a", isAlive: true }]);
    const newM = makeModel([{ id: "a", isAlive: false }]);
    const result = diff.computeGraphDiff(old, newM);
    assert.strictEqual(result.modifiedNodes.length, 1);
    assert.strictEqual(result.modifiedNodes[0].changes.isAlive, false);
  });

  // 7. Added edge detected
  it("detects added edge", () => {
    const old = makeModel([{ id: "a" }, { id: "b" }], []);
    const newM = makeModel([{ id: "a" }, { id: "b" }], [{ id: "e0", source: "a", target: "b" }]);
    const result = diff.computeGraphDiff(old, newM);
    assert.strictEqual(result.addedEdges.length, 1);
    assert.strictEqual(result.addedEdges[0].id, "e0");
  });

  // 8. Removed edge detected
  it("detects removed edge", () => {
    const old = makeModel([{ id: "a" }, { id: "b" }], [{ id: "e0", source: "a", target: "b" }]);
    const newM = makeModel([{ id: "a" }, { id: "b" }], []);
    const result = diff.computeGraphDiff(old, newM);
    assert.strictEqual(result.removedEdgeIds.length, 1);
    assert.strictEqual(result.removedEdgeIds[0], "e0");
  });

  // 9. Unchanged node not in modifiedNodes
  it("unchanged node not in modifiedNodes", () => {
    const old = makeModel([{ id: "a", category: "Owned" }, { id: "b", category: "SharedRef" }]);
    const newM = makeModel([{ id: "a", category: "Owned" }, { id: "b", category: "MutableRef" }]);
    const result = diff.computeGraphDiff(old, newM);
    assert.strictEqual(result.modifiedNodes.length, 1);
    assert.strictEqual(result.modifiedNodes[0].id, "b");
  });

  // 10. Performance: diff computation is fast
  it("diff computation is fast for 50 nodes", () => {
    const nodes = Array.from({ length: 50 }, (_, i) => ({ id: `n${i}` }));
    const edges = Array.from({ length: 40 }, (_, i) => ({ id: `e${i}`, source: `n${i}`, target: `n${i + 1}` }));
    const old = makeModel(nodes, edges);
    // Add one node, remove one node
    const newNodes = [...nodes.slice(1), { id: "n50" }];
    const newM = makeModel(newNodes, edges);
    const start = Date.now();
    const result = diff.computeGraphDiff(old, newM);
    const elapsed = Date.now() - start;
    assert.ok(elapsed < 5, `Diff should be < 5ms, got ${elapsed}ms`);
    assert.ok(result.hasChanges);
  });

  // 11. Comment-only edit (same graph) produces no diff
  it("same graph data produces hasChanges=false", () => {
    const model = makeModel([{ id: "a" }, { id: "b" }], [{ id: "e0", source: "a", target: "b" }]);
    const copy = JSON.parse(JSON.stringify(model));
    const result = diff.computeGraphDiff(model, copy);
    assert.strictEqual(result.hasChanges, false);
  });

  // 12. Size change detected
  it("detects size change", () => {
    const old = makeModel([{ id: "x", size: 1 }]);
    const newM = makeModel([{ id: "x", size: 3 }]);
    const result = diff.computeGraphDiff(old, newM);
    assert.strictEqual(result.modifiedNodes.length, 1);
    assert.strictEqual(result.modifiedNodes[0].changes.size, 3);
  });
});
