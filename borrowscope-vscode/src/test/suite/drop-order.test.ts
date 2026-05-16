import * as assert from "assert";
import * as path from "path";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("12.8 Drop Order Visualization", () => {
  let dropOrder: any;

  before(() => {
    dropOrder = require(path.join(ROOT, "out", "drop-order"));
  });

  // === buildDropOrder ===

  it("returns empty for no events", () => {
    assert.deepStrictEqual(dropOrder.buildDropOrder([]), []);
  });

  it("builds drop order from Drop events", () => {
    const events = [
      { New: { timestamp: 10, var_name: "a", var_id: "a_1", type_name: "String" } },
      { New: { timestamp: 20, var_name: "b", var_id: "b_1", type_name: "String" } },
      { New: { timestamp: 30, var_name: "c", var_id: "c_1", type_name: "String" } },
      { Drop: { timestamp: 100, var_id: "c_1", location: "src/main.rs:10:1" } },
      { Drop: { timestamp: 110, var_id: "b_1", location: "src/main.rs:10:1" } },
      { Drop: { timestamp: 120, var_id: "a_1", location: "src/main.rs:10:1" } },
    ];
    const drops = dropOrder.buildDropOrder(events);
    assert.strictEqual(drops.length, 3);
    assert.strictEqual(drops[0].var_name, "c");
    assert.strictEqual(drops[0].order, 1);
    assert.strictEqual(drops[1].var_name, "b");
    assert.strictEqual(drops[1].order, 2);
    assert.strictEqual(drops[2].var_name, "a");
    assert.strictEqual(drops[2].order, 3);
  });

  it("computes lifetime_ns from New to Drop", () => {
    const events = [
      { New: { timestamp: 1000, var_name: "x", var_id: "x_1", type_name: "i32" } },
      { Drop: { timestamp: 5000, var_id: "x_1" } },
    ];
    const drops = dropOrder.buildDropOrder(events);
    assert.strictEqual(drops[0].lifetime_ns, 4000);
  });

  it("lifetime_ns is -1 if no creation found", () => {
    const events = [{ Drop: { timestamp: 100, var_id: "unknown_1" } }];
    const drops = dropOrder.buildDropOrder(events);
    assert.strictEqual(drops[0].lifetime_ns, -1);
  });

  it("tracks scope depth from RegionEnter/Exit", () => {
    const events = [
      { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
      { RegionEnter: { timestamp: 20, region_id: "r_1", name: "inner", location: "src/main.rs:3:5" } },
      { New: { timestamp: 30, var_name: "y", var_id: "y_1", type_name: "i32" } },
      { Drop: { timestamp: 40, var_id: "y_1" } },
      { RegionExit: { timestamp: 50, region_id: "r_1", location: "src/main.rs:5:5" } },
      { Drop: { timestamp: 60, var_id: "x_1" } },
    ];
    const drops = dropOrder.buildDropOrder(events);
    assert.strictEqual(drops[0].scope_depth, 1); // y dropped inside region
    assert.strictEqual(drops[1].scope_depth, 0); // x dropped outside
  });

  it("tracks function name from FnEnter/FnExit", () => {
    const events = [
      { FnEnter: { timestamp: 5, fn_id: "f1", fn_name: "main", location: "src/main.rs:1:1" } },
      { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
      { Drop: { timestamp: 100, var_id: "x_1" } },
      { FnExit: { timestamp: 110, fn_id: "f1", fn_name: "main", location: "src/main.rs:10:1" } },
    ];
    const drops = dropOrder.buildDropOrder(events);
    assert.strictEqual(drops[0].function_name, "main");
  });

  it("tracks LockGuardDrop as a drop", () => {
    const events = [
      { LockGuardAcquire: { timestamp: 10, guard_id: "g_1", lock_id: "lk_1", lock_type: "Mutex", location: "src/main.rs:5:5" } },
      { LockGuardDrop: { timestamp: 50, guard_id: "g_1", location: "src/main.rs:8:5" } },
    ];
    const drops = dropOrder.buildDropOrder(events);
    assert.strictEqual(drops.length, 1);
    assert.ok(drops[0].var_name.includes("g_1"));
  });

  it("handles RcNew/ArcNew/BoxNew creation timestamps", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc", var_id: "rc_1", type_name: "Rc<i32>", strong_count: 1, weak_count: 0 } },
      { BoxNew: { timestamp: 20, var_name: "b", var_id: "b_1", type_name: "Box<i32>", location: "src/main.rs:4:5" } },
      { Drop: { timestamp: 100, var_id: "rc_1" } },
      { Drop: { timestamp: 110, var_id: "b_1" } },
    ];
    const drops = dropOrder.buildDropOrder(events);
    assert.strictEqual(drops[0].lifetime_ns, 90);  // 100 - 10
    assert.strictEqual(drops[1].lifetime_ns, 90);  // 110 - 20
  });

  it("parses location into SourceLocation", () => {
    const events = [
      { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
      { Drop: { timestamp: 100, var_id: "x_1", location: "src/main.rs:15:3" } },
    ];
    const drops = dropOrder.buildDropOrder(events);
    assert.deepStrictEqual(drops[0].location, { file: "src/main.rs", line: 15, column: 3 });
  });

  // === analyzeDropOrder ===

  it("detects LIFO order (correct)", () => {
    const events = [
      { FnEnter: { timestamp: 5, fn_id: "f1", fn_name: "main", location: "src/main.rs:1:1" } },
      { New: { timestamp: 10, var_name: "a", var_id: "a_1", type_name: "String" } },
      { New: { timestamp: 20, var_name: "b", var_id: "b_1", type_name: "String" } },
      { New: { timestamp: 30, var_name: "c", var_id: "c_1", type_name: "String" } },
      { Drop: { timestamp: 100, var_id: "c_1" } },
      { Drop: { timestamp: 110, var_id: "b_1" } },
      { Drop: { timestamp: 120, var_id: "a_1" } },
      { FnExit: { timestamp: 130, fn_id: "f1", fn_name: "main", location: "src/main.rs:10:1" } },
    ];
    const analysis = dropOrder.analyzeDropOrder(events, "main");
    assert.strictEqual(analysis.is_lifo, true);
    assert.strictEqual(analysis.violations.length, 0);
  });

  it("detects non-LIFO order (early drop)", () => {
    const events = [
      { FnEnter: { timestamp: 5, fn_id: "f1", fn_name: "main", location: "src/main.rs:1:1" } },
      { New: { timestamp: 10, var_name: "a", var_id: "a_1", type_name: "String" } },
      { New: { timestamp: 20, var_name: "b", var_id: "b_1", type_name: "String" } },
      // a dropped before b — non-LIFO
      { Drop: { timestamp: 100, var_id: "a_1" } },
      { Drop: { timestamp: 110, var_id: "b_1" } },
      { FnExit: { timestamp: 120, fn_id: "f1", fn_name: "main", location: "src/main.rs:10:1" } },
    ];
    const analysis = dropOrder.analyzeDropOrder(events, "main");
    assert.strictEqual(analysis.is_lifo, false);
    assert.ok(analysis.violations.length > 0);
  });

  it("analyzeDropOrder without function filter", () => {
    const events = [
      { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } },
      { Drop: { timestamp: 100, var_id: "x_1" } },
    ];
    const analysis = dropOrder.analyzeDropOrder(events);
    assert.strictEqual(analysis.function_name, null);
    assert.strictEqual(analysis.drops.length, 1);
  });

  // === getDropOrderForVar ===

  it("returns drop order for a specific var", () => {
    const events = [
      { New: { timestamp: 10, var_name: "a", var_id: "a_1", type_name: "i32" } },
      { New: { timestamp: 20, var_name: "b", var_id: "b_1", type_name: "i32" } },
      { Drop: { timestamp: 100, var_id: "b_1" } },
      { Drop: { timestamp: 110, var_id: "a_1" } },
    ];
    assert.strictEqual(dropOrder.getDropOrderForVar(events, "b_1"), 1);
    assert.strictEqual(dropOrder.getDropOrderForVar(events, "a_1"), 2);
  });

  it("returns -1 for var that was never dropped", () => {
    const events = [{ New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } }];
    assert.strictEqual(dropOrder.getDropOrderForVar(events, "x_1"), -1);
  });

  // === longestLived / shortestLived ===

  it("longestLived returns top N by lifetime", () => {
    const events = [
      { New: { timestamp: 10, var_name: "short", var_id: "s_1", type_name: "i32" } },
      { New: { timestamp: 20, var_name: "long", var_id: "l_1", type_name: "String" } },
      { Drop: { timestamp: 30, var_id: "s_1" } },  // 20ns
      { Drop: { timestamp: 1000, var_id: "l_1" } }, // 980ns
    ];
    const longest = dropOrder.longestLived(events, 1);
    assert.strictEqual(longest.length, 1);
    assert.strictEqual(longest[0].var_name, "long");
    assert.strictEqual(longest[0].lifetime_ns, 980);
  });

  it("shortestLived returns bottom N by lifetime", () => {
    const events = [
      { New: { timestamp: 10, var_name: "short", var_id: "s_1", type_name: "i32" } },
      { New: { timestamp: 20, var_name: "long", var_id: "l_1", type_name: "String" } },
      { Drop: { timestamp: 30, var_id: "s_1" } },
      { Drop: { timestamp: 1000, var_id: "l_1" } },
    ];
    const shortest = dropOrder.shortestLived(events, 1);
    assert.strictEqual(shortest.length, 1);
    assert.strictEqual(shortest[0].var_name, "short");
  });

  it("longestLived returns empty for no drops", () => {
    assert.deepStrictEqual(dropOrder.longestLived([], 5), []);
  });
});
