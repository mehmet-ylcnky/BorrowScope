import * as assert from "assert";
import * as path from "path";

const ROOT = path.resolve(__dirname, "..", "..", "..");

describe("12.4 Merge Static + Runtime Views", () => {
  let merge: any;

  before(() => {
    merge = require(path.join(ROOT, "out", "merge-views"));
  });

  const staticVars = [
    { name: "data", line: 3, type_display: "Vec<i32>", ownership_category: "Owned", is_copy: false },
    { name: "r", line: 5, type_display: "&Vec<i32>", ownership_category: "SharedRef", is_copy: true },
    { name: "rc", line: 7, type_display: "Rc<i32>", ownership_category: "Rc", is_copy: false },
    { name: "unused", line: 20, type_display: "i32", ownership_category: "Owned", is_copy: true },
  ];

  // === mergeViews basic ===

  it("mergeViews returns MergedVariable array", () => {
    const events = [{ New: { timestamp: 10, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } }];
    const merged = merge.mergeViews(staticVars, events);
    assert.ok(Array.isArray(merged));
    assert.ok(merged.length > 0);
  });

  it("mergeViews includes static_only for unmatched static vars", () => {
    const events = [{ New: { timestamp: 10, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } }];
    const merged = merge.mergeViews(staticVars, events);
    const staticOnly = merged.filter((m: any) => m.agreement === "static_only");
    assert.ok(staticOnly.length > 0);
  });

  it("mergeViews includes runtime_only for unmatched runtime vars", () => {
    const events = [{ New: { timestamp: 10, var_name: "unknown_var", var_id: "uv_1", type_name: "Foo" } }];
    const merged = merge.mergeViews(staticVars, events);
    const runtimeOnly = merged.find((m: any) => m.agreement === "runtime_only");
    assert.ok(runtimeOnly);
  });

  it("mergeViews computes match for agreeing data", () => {
    const events = [
      { New: { timestamp: 10, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } },
      { Drop: { timestamp: 100, var_id: "data_1" } },
    ];
    const merged = merge.mergeViews(staticVars, events);
    const dataVar = merged.find((m: any) => m.name === "data" && m.var_id === "data_1");
    assert.strictEqual(dataVar.agreement, "match");
  });

  // === RuntimeInfo computation ===

  it("computes actual_lifetime_ns from New to Drop", () => {
    const events = [
      { New: { timestamp: 1000, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } },
      { Drop: { timestamp: 5000, var_id: "data_1" } },
    ];
    const merged = merge.mergeViews(staticVars, events);
    const dataVar = merged.find((m: any) => m.var_id === "data_1");
    assert.strictEqual(dataVar.runtime_info.actual_lifetime_ns, 4000);
  });

  it("actual_lifetime_ns is -1 when not dropped", () => {
    const events = [{ New: { timestamp: 1000, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } }];
    const merged = merge.mergeViews(staticVars, events);
    const dataVar = merged.find((m: any) => m.var_id === "data_1");
    assert.strictEqual(dataVar.runtime_info.actual_lifetime_ns, -1);
  });

  it("counts borrows correctly", () => {
    const events = [
      { New: { timestamp: 10, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } },
      { Borrow: { timestamp: 20, borrower_name: "r1", borrower_id: "r1_1", owner_id: "data_1", mutable: false } },
      { Borrow: { timestamp: 30, borrower_name: "r2", borrower_id: "r2_1", owner_id: "data_1", mutable: true } },
      { Drop: { timestamp: 100, var_id: "data_1" } },
    ];
    const merged = merge.mergeViews(staticVars, events);
    const dataVar = merged.find((m: any) => m.var_id === "data_1");
    assert.strictEqual(dataVar.runtime_info.actual_borrow_count, 2);
    assert.strictEqual(dataVar.runtime_info.actual_mut_borrow_count, 1);
  });

  it("detects move", () => {
    const events = [
      { New: { timestamp: 10, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } },
      { Move: { timestamp: 50, from_id: "data_1", to_name: "other", to_id: "other_1" } },
    ];
    const merged = merge.mergeViews(staticVars, events);
    const dataVar = merged.find((m: any) => m.var_id === "data_1");
    assert.strictEqual(dataVar.runtime_info.was_actually_moved, true);
    assert.strictEqual(dataVar.runtime_info.move_destination, "other");
  });

  it("tracks ref_count_peak for Rc", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc", var_id: "rc_1", type_name: "Rc<i32>", strong_count: 1, weak_count: 0 } },
      { RcClone: { timestamp: 20, var_name: "rc2", var_id: "rc2_1", source_id: "rc_1", strong_count: 2, weak_count: 0 } },
      { RcClone: { timestamp: 30, var_name: "rc3", var_id: "rc3_1", source_id: "rc_1", strong_count: 3, weak_count: 0 } },
      { Drop: { timestamp: 100, var_id: "rc_1" } },
    ];
    const merged = merge.mergeViews(staticVars, events);
    const rcVar = merged.find((m: any) => m.var_id === "rc_1");
    assert.strictEqual(rcVar.runtime_info.ref_count_peak, 3);
    assert.strictEqual(rcVar.runtime_info.clone_count, 2);
  });

  it("tracks drop_order", () => {
    const events = [
      { New: { timestamp: 10, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } },
      { RcNew: { timestamp: 20, var_name: "rc", var_id: "rc_1", type_name: "Rc<i32>", strong_count: 1, weak_count: 0 } },
      { Drop: { timestamp: 100, var_id: "rc_1" } },
      { Drop: { timestamp: 200, var_id: "data_1" } },
    ];
    const merged = merge.mergeViews(staticVars, events);
    const rcVar = merged.find((m: any) => m.var_id === "rc_1");
    const dataVar = merged.find((m: any) => m.var_id === "data_1");
    assert.strictEqual(rcVar.runtime_info.drop_order, 0);
    assert.strictEqual(dataVar.runtime_info.drop_order, 1);
  });

  it("counts unsafe accesses", () => {
    const statics = [{ name: "ptr", line: 3, type_display: "*const i32", ownership_category: "RawPtr", is_copy: false }];
    const events = [
      { RawPtrCreated: { timestamp: 10, var_name: "ptr", var_id: "ptr_1", ptr_type: "*const i32", address: 12345, location: "src/main.rs:3:5" } },
      { RawPtrDeref: { timestamp: 20, ptr_id: "ptr_1", location: "src/main.rs:5:5", is_write: false } },
      { Drop: { timestamp: 100, var_id: "ptr_1" } },
    ];
    const merged = merge.mergeViews(statics, events);
    const ptrVar = merged.find((m: any) => m.var_id === "ptr_1");
    assert.strictEqual(ptrVar.runtime_info.unsafe_accesses, 1);
  });

  it("tracks event_count", () => {
    const events = [
      { New: { timestamp: 10, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } },
      { Borrow: { timestamp: 20, borrower_name: "r", borrower_id: "r_1", owner_id: "data_1", mutable: false } },
      { Drop: { timestamp: 100, var_id: "data_1" } },
    ];
    const merged = merge.mergeViews(staticVars, events);
    const dataVar = merged.find((m: any) => m.var_id === "data_1");
    assert.strictEqual(dataVar.runtime_info.event_count, 3);
  });

  // === Divergence detection ===

  it("detects Rc leak (never dropped, ref_count > 0)", () => {
    const events = [
      { RcNew: { timestamp: 10, var_name: "rc", var_id: "rc_1", type_name: "Rc<i32>", strong_count: 1, weak_count: 0 } },
      { RcClone: { timestamp: 20, var_name: "rc2", var_id: "rc2_1", source_id: "rc_1", strong_count: 2, weak_count: 0 } },
      // No Drop — leaked!
    ];
    const merged = merge.mergeViews(staticVars, events);
    const rcVar = merged.find((m: any) => m.var_id === "rc_1");
    assert.strictEqual(rcVar.agreement, "diverge");
    assert.ok(rcVar.divergences.some((d: any) => d.kind === "rc_leak"));
  });

  it("detects missing drop for non-Copy owned var", () => {
    const statics = [{ name: "s", line: 3, type_display: "String", ownership_category: "Owned", is_copy: false }];
    const events = [
      { New: { timestamp: 10, var_name: "s", var_id: "s_1", type_name: "String" } },
      // No Drop, no Move
    ];
    const merged = merge.mergeViews(statics, events);
    const sVar = merged.find((m: any) => m.var_id === "s_1");
    assert.strictEqual(sVar.agreement, "diverge");
    assert.ok(sVar.divergences.some((d: any) => d.kind === "missing_drop"));
  });

  it("detects async borrow held across await", () => {
    const events = [
      { New: { timestamp: 10, var_name: "data", var_id: "data_1", type_name: "Vec<i32>" } },
      { Borrow: { timestamp: 20, borrower_name: "r", borrower_id: "r_1", owner_id: "data_1", mutable: false } },
      { AwaitStart: { timestamp: 30, await_id: "aw_1", future_name: "fetch", location: "src/main.rs:6:5", live_variables: ["r"] } },
      { AwaitEnd: { timestamp: 500, await_id: "aw_1", location: "src/main.rs:6:5" } },
      { Drop: { timestamp: 600, var_id: "data_1" } },
    ];
    const merged = merge.mergeViews(staticVars, events);
    const dataVar = merged.find((m: any) => m.var_id === "data_1");
    assert.strictEqual(dataVar.agreement, "diverge");
    assert.ok(dataVar.divergences.some((d: any) => d.kind === "async_borrow_held"));
    assert.strictEqual(dataVar.runtime_info.await_crossings.length, 1);
    assert.strictEqual(dataVar.runtime_info.await_crossings[0].duration_ns, 470);
  });

  it("detects unsafe hidden info", () => {
    const statics = [{ name: "v", line: 3, type_display: "Vec<u8>", ownership_category: "Owned", is_copy: false }];
    const events = [
      { New: { timestamp: 10, var_name: "v", var_id: "v_1", type_name: "Vec<u8>" } },
      { UnsafeBlockEnter: { timestamp: 20, block_id: "ub_1", location: "src/main.rs:5:5" } },
      { RawPtrDeref: { timestamp: 30, ptr_id: "v_1", location: "src/main.rs:6:5", is_write: true } },
      { Drop: { timestamp: 100, var_id: "v_1" } },
    ];
    const merged = merge.mergeViews(statics, events);
    const vVar = merged.find((m: any) => m.var_id === "v_1");
    assert.ok(vVar.divergences.some((d: any) => d.kind === "unsafe_hidden"));
  });

  it("no divergence for normal owned var with drop", () => {
    const statics = [{ name: "x", line: 3, type_display: "String", ownership_category: "Owned", is_copy: false }];
    const events = [
      { New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "String" } },
      { Drop: { timestamp: 100, var_id: "x_1" } },
    ];
    const merged = merge.mergeViews(statics, events);
    const xVar = merged.find((m: any) => m.var_id === "x_1");
    assert.strictEqual(xVar.agreement, "match");
    assert.strictEqual(xVar.divergences.length, 0);
  });

  it("no divergence for Copy type without drop", () => {
    const statics = [{ name: "n", line: 2, type_display: "i32", ownership_category: "Owned", is_copy: true }];
    const events = [
      { New: { timestamp: 10, var_name: "n", var_id: "n_1", type_name: "i32" } },
      // No drop — fine for Copy types
    ];
    const merged = merge.mergeViews(statics, events);
    const nVar = merged.find((m: any) => m.var_id === "n_1");
    // Copy types don't need explicit drop
    assert.strictEqual(nVar.divergences.filter((d: any) => d.kind === "missing_drop").length, 0);
  });

  // === mergeSummary ===

  it("mergeSummary counts agreements", () => {
    const merged = [
      { agreement: "match" },
      { agreement: "match" },
      { agreement: "diverge" },
      { agreement: "static_only" },
      { agreement: "runtime_only" },
    ];
    const s = merge.mergeSummary(merged);
    assert.strictEqual(s.total, 5);
    assert.strictEqual(s.match, 2);
    assert.strictEqual(s.diverge, 1);
    assert.strictEqual(s.static_only, 1);
    assert.strictEqual(s.runtime_only, 1);
  });

  // === Edge cases ===

  it("handles empty events", () => {
    const merged = merge.mergeViews(staticVars, []);
    assert.ok(merged.length > 0); // static_only entries
    assert.ok(merged.every((m: any) => m.agreement === "static_only"));
  });

  it("handles empty static vars", () => {
    const events = [{ New: { timestamp: 10, var_name: "x", var_id: "x_1", type_name: "i32" } }];
    const merged = merge.mergeViews([], events);
    assert.strictEqual(merged.length, 1);
    assert.strictEqual(merged[0].agreement, "runtime_only");
  });

  it("handles both empty", () => {
    const merged = merge.mergeViews([], []);
    assert.strictEqual(merged.length, 0);
  });

  it("handles Arc events same as Rc", () => {
    const statics = [{ name: "a", line: 3, type_display: "Arc<Mutex<i32>>", ownership_category: "Arc", is_copy: false }];
    const events = [
      { ArcNew: { timestamp: 10, var_name: "a", var_id: "a_1", type_name: "Arc<Mutex<i32>>", strong_count: 1, weak_count: 0 } },
      { ArcClone: { timestamp: 20, var_name: "a2", var_id: "a2_1", source_id: "a_1", strong_count: 2, weak_count: 0 } },
      { Drop: { timestamp: 100, var_id: "a_1" } },
    ];
    const merged = merge.mergeViews(statics, events);
    const aVar = merged.find((m: any) => m.var_id === "a_1");
    assert.strictEqual(aVar.runtime_info.ref_count_peak, 2);
    assert.strictEqual(aVar.runtime_info.clone_count, 1);
  });

  it("handles WeakNew with weak_count tracking", () => {
    const statics = [{ name: "w", line: 5, type_display: "Weak<i32>", ownership_category: "Weak", is_copy: false }];
    const events = [
      { WeakNew: { timestamp: 10, var_name: "w", var_id: "w_1", source_id: "rc_1", weak_count: 1, location: "src/main.rs:5:5" } },
      { Drop: { timestamp: 100, var_id: "w_1" } },
    ];
    const merged = merge.mergeViews(statics, events);
    const wVar = merged.find((m: any) => m.var_id === "w_1");
    assert.strictEqual(wVar.runtime_info.weak_count_peak, 1);
  });

  it("handles Box events", () => {
    const statics = [{ name: "b", line: 4, type_display: "Box<String>", ownership_category: "Owned", is_copy: false }];
    const events = [
      { BoxNew: { timestamp: 10, var_name: "b", var_id: "b_1", type_name: "Box<String>", location: "src/main.rs:4:5" } },
      { Drop: { timestamp: 100, var_id: "b_1" } },
    ];
    const merged = merge.mergeViews(statics, events);
    const bVar = merged.find((m: any) => m.var_id === "b_1");
    assert.strictEqual(bVar.agreement, "match");
  });
});
